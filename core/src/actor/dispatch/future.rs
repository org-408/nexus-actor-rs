use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use crate::actor::actor::ExtendedPid;
use crate::actor::actor_system::ActorSystem;
use crate::actor::dispatch::Runnable;
use crate::actor::message::Message;
use crate::actor::message::MessageHandle;
use crate::actor::metrics::metrics_impl::{Metrics, EXTENSION_ID};
use crate::actor::process::{Process, ProcessHandle};
use crate::generated::actor::DeadLetterResponse;
use crate::metrics::ActorMetrics;
use async_trait::async_trait;
use futures::future::BoxFuture;
use nexus_actor_message_derive_rs::Message;
use opentelemetry::KeyValue;
use thiserror::Error;
use tokio::sync::{Notify, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Message, Error)]
pub enum ActorFutureError {
  #[error("future: timeout")]
  TimeoutError,
  #[error("future: dead letter")]
  DeadLetterError,
}

#[derive(Clone)]
struct Completion(
  Arc<dyn Fn(Option<MessageHandle>, Option<ActorFutureError>) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
);

unsafe impl Send for Completion {}
unsafe impl Sync for Completion {}

impl Completion {
  fn new<F, Fut>(f: F) -> Self
  where
    F: Fn(Option<MessageHandle>, Option<ActorFutureError>) -> Fut + Send + Sync + 'static,
    Fut: core::future::Future<Output = ()> + Send + 'static, {
    Self(Arc::new(move |message, error| {
      Box::pin(f(message, error)) as BoxFuture<'static, ()>
    }))
  }

  async fn run(&self, result: Option<MessageHandle>, error: Option<ActorFutureError>) {
    (self.0)(result, error).await
  }
}

impl Debug for Completion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Completion")
  }
}

#[derive(Debug, Clone)]
pub struct ActorFutureProcess {
  future: Arc<RwLock<ActorFuture>>,
}

impl ActorFutureProcess {
  pub async fn new(system: ActorSystem, duration: Duration) -> Arc<Self> {
    let inner = Arc::new(RwLock::new(ActorFutureInner {
      actor_system: system.clone(),
      pid: None,
      done: false,
      result: None,
      error: None,
      pipes: Vec::new(),
      completions: Vec::new(),
    }));
    let notify = Arc::new(Notify::new());

    let future = ActorFuture { inner, notify };

    let future_process = Arc::new(ActorFutureProcess {
      future: Arc::new(RwLock::new(future.clone())),
    });

    let process_registry = system.get_process_registry().await;
    let id = process_registry.next_id();

    let (pid, ok) = process_registry
      .add_process(
        ProcessHandle::new_arc(future_process.clone()),
        &format!("future_{}", id),
      )
      .await;
    if !ok {
      tracing::error!("failed to register future process: pid = {}", pid);
    }

    future_process
      .metrics_foreach(|am, _| {
        let am = am.clone();
        async move { am.increment_futures_started_count().await }
      })
      .await;

    future_process.set_pid(pid).await;

    if duration > Duration::from_secs(0) {
      let future_process_clone = Arc::clone(&future_process);

      system
        .get_config()
        .await
        .system_dispatcher
        .schedule(Runnable::new(move || async move {
          let future = future_process_clone.get_future().await;

          tokio::select! {
              _ = future.notify.notified() => {
                tracing::debug!("Future completed");
              }
              _ = tokio::time::sleep(duration) => {
                  tracing::debug!("Future timed out");
                  future_process_clone.handle_timeout().await;
              }
          }
        }))
        .await;
    }

    future_process
  }

  async fn metrics_foreach<F, Fut>(&self, f: F)
  where
    F: Fn(&ActorMetrics, &Metrics) -> Fut,
    Fut: std::future::Future<Output = ()>, {
    if self.get_actor_system().await.get_config().await.is_metrics_enabled() {
      if let Some(extension_arc) = self
        .get_actor_system()
        .await
        .get_extensions()
        .await
        .get(*EXTENSION_ID)
        .await
      {
        let mut extension = extension_arc.lock().await;
        if let Some(m) = extension.as_any_mut().downcast_mut::<Metrics>() {
          m.foreach(f).await;
        }
      }
    }
  }

  async fn get_actor_system(&self) -> ActorSystem {
    let future = self.future.read().await;
    let inner = future.inner.read().await;
    inner.actor_system.clone()
  }

  pub async fn set_pid(&self, pid: ExtendedPid) {
    let mut future_mg = self.future.write().await;
    future_mg.set_pid(pid).await;
  }

  pub async fn get_pid(&self) -> ExtendedPid {
    let future_mg = self.future.read().await;
    future_mg.get_pid().await
  }

  pub async fn get_future(&self) -> ActorFuture {
    let future_mg = self.future.read().await;
    future_mg.clone()
  }

  pub async fn is_empty(&self) -> bool {
    let future_mg = self.future.read().await;
    let inner = future_mg.inner.read().await;
    !inner.done
  }

  pub async fn pipe_to(&self, pid: ExtendedPid) {
    let future_mg = self.future.read().await;
    future_mg.pipe_to(pid).await;
  }

  pub async fn result(&self) -> Result<MessageHandle, ActorFutureError> {
    let future_mg = self.future.read().await;
    future_mg.result().await
  }

  pub async fn complete(&self, result: MessageHandle) {
    let future_mg = self.future.read().await;
    future_mg.complete(result).await;
  }

  pub async fn fail(&self, error: ActorFutureError) {
    let future_mg = self.future.read().await;
    future_mg.fail(error).await;
  }

  async fn handle_timeout(&self) {
    let error = ActorFutureError::TimeoutError;
    self.fail(error.clone()).await;

    {
      let future = self.future.read().await;
      let mut inner = future.inner.write().await;
      for pipe in &inner.pipes {
        pipe
          .send_user_message(inner.actor_system.clone(), MessageHandle::new(error.clone()))
          .await;
      }
      inner.pipes.clear();
    }
  }

  async fn instrument(&self, future: ActorFuture) {
    self
      .metrics_foreach(|am, _| {
        let cloned_am = am.clone();
        let cloned_future = future.clone();
        async move {
          if {
            let actor_future_inner = cloned_future.inner.read().await;
            actor_future_inner.error.is_none()
          } {
            cloned_am
              .increment_futures_completed_count_with_opts(&[KeyValue::new(
                "address",
                self.get_actor_system().await.get_address().await,
              )])
              .await
          } else {
            cloned_am
              .increment_futures_timed_out_count_with_opts(&[KeyValue::new(
                "address",
                self.get_actor_system().await.get_address().await,
              )])
              .await
          }
        }
      })
      .await;
    future.instrument().await;
  }
}

#[async_trait]
impl Process for ActorFutureProcess {
  async fn send_user_message(&self, _: Option<&ExtendedPid>, message_handle: MessageHandle) {
    let cloned_self = self.clone();
    let future = self.future.read().await.clone();
    let dispatcher = {
      let mg = future.inner.read().await;
      mg.actor_system.get_config().await.system_dispatcher.clone()
    };
    dispatcher
      .schedule(Runnable::new(move || {
        let future = future.clone();
        let cloned_self = cloned_self.clone();
        async move {
          if message_handle.to_typed::<DeadLetterResponse>().is_some() {
            future.fail(ActorFutureError::DeadLetterError).await;
          } else {
            future.complete(message_handle.clone()).await;
          }
          cloned_self.instrument(future).await;
        }
      }))
      .await;
  }

  async fn send_system_message(&self, _: &ExtendedPid, message_handle: MessageHandle) {
    let cloned_self = self.clone();
    let future = self.future.read().await.clone();
    let dispatcher = {
      let mg = future.inner.read().await;
      mg.actor_system.get_config().await.system_dispatcher.clone()
    };
    dispatcher
      .schedule(Runnable::new(move || {
        let future = future.clone();
        let cloned_self = cloned_self.clone();
        async move {
          future.complete(message_handle.clone()).await;
          cloned_self.instrument(future).await;
        }
      }))
      .await;
  }

  async fn stop(&self, _pid: &ExtendedPid) {}

  fn set_dead(&self) {}

  fn as_any(&self) -> &dyn Any {
    self
  }
}

#[derive(Debug)]
struct ActorFutureInner {
  actor_system: ActorSystem,
  pid: Option<ExtendedPid>,
  done: bool,
  result: Option<MessageHandle>,
  error: Option<ActorFutureError>,
  pipes: Vec<ExtendedPid>,
  completions: Vec<Completion>,
}

static_assertions::assert_impl_all!(ActorFutureInner: Send, Sync);
#[derive(Debug, Clone)]
pub struct ActorFuture {
  inner: Arc<RwLock<ActorFutureInner>>,
  notify: Arc<Notify>,
}

static_assertions::assert_impl_all!(ActorFuture: Send, Sync);

impl ActorFuture {
  pub async fn result(&self) -> Result<MessageHandle, ActorFutureError> {
    loop {
      {
        let inner = self.inner.read().await;
        if inner.done {
          return if let Some(error) = &inner.error {
            Err(error.clone())
          } else {
            Ok(inner.result.as_ref().unwrap().clone())
          };
        }
      }
      self.notify.notified().await;
    }
  }

  pub async fn wait(&self) -> Option<ActorFutureError> {
    self.result().await.err()
  }

  pub async fn set_pid(&mut self, pid: ExtendedPid) {
    let mut inner = self.inner.write().await;
    inner.pid = Some(pid);
  }

  pub async fn get_pid(&self) -> ExtendedPid {
    let inner = self.inner.read().await;
    inner.pid.clone().expect("pid not set")
  }

  pub async fn pipe_to(&self, pid: ExtendedPid) {
    let mut inner = self.inner.write().await;
    inner.pipes.push(pid);
    if inner.done {
      self.send_to_pipes(&mut inner).await;
    }
  }

  async fn send_to_pipes(&self, inner: &mut ActorFutureInner) {
    let message = if let Some(error) = &inner.error {
      MessageHandle::new(error.clone())
    } else {
      inner.result.as_ref().unwrap().clone()
    };

    for process in &inner.pipes {
      process
        .send_user_message(inner.actor_system.clone(), message.clone())
        .await;
    }

    inner.pipes.clear();
  }

  pub async fn complete(&self, result: MessageHandle) {
    let mut inner = self.inner.write().await;
    if !inner.done {
      inner.result = Some(result);
      inner.done = true;
      self.send_to_pipes(&mut inner).await;
      self.run_completions(&mut inner).await;
      self.notify.notify_waiters();
    }
  }

  pub async fn fail(&self, error: ActorFutureError) {
    let mut inner = self.inner.write().await;
    if !inner.done {
      inner.error = Some(error);
      inner.done = true;
      self.send_to_pipes(&mut inner).await;
      self.run_completions(&mut inner).await;
      self.notify.notify_waiters();
    }
  }

  pub async fn continue_with<F, Fut>(&self, continuation: F)
  where
    F: Fn(Option<MessageHandle>, Option<ActorFutureError>) -> Fut + Send + Sync + 'static,
    Fut: core::future::Future<Output = ()> + Send + 'static, {
    let mut inner = self.inner.write().await;
    if inner.done {
      continuation(inner.result.clone(), inner.error.clone()).await;
    } else {
      inner.completions.push(Completion::new(continuation));
    }
  }

  async fn run_completions(&self, inner: &mut ActorFutureInner) {
    for completion in inner.completions.drain(..) {
      completion.run(inner.result.clone(), inner.error.clone()).await;
    }
  }

  async fn instrument(&self) {
    // Here you would implement your metrics logging
    // This is a placeholder for the actual implementation
  }

  async fn get_actor_system(&self) -> ActorSystem {
    let mg = self.inner.read().await;
    mg.actor_system.clone()
  }

  async fn metrics_foreach<F, Fut>(&self, f: F)
  where
    F: Fn(&ActorMetrics, &Metrics) -> Fut,
    Fut: std::future::Future<Output = ()>, {
    if self.get_actor_system().await.get_config().await.is_metrics_enabled() {
      if let Some(extension_arc) = self
        .get_actor_system()
        .await
        .get_extensions()
        .await
        .get(*EXTENSION_ID)
        .await
      {
        let mut extension = extension_arc.lock().await;
        if let Some(m) = extension.as_any_mut().downcast_mut::<Metrics>() {
          m.foreach(f).await;
        }
      }
    }
  }
}
