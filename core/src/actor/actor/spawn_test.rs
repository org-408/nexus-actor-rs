#![cfg(test)]
mod tests {
  use std::env;
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::sync::Arc;

  use async_trait::async_trait;
  use tokio::sync::Notify;
  use tracing_subscriber::EnvFilter;

  use crate::actor::actor::actor::Actor;
  use crate::actor::actor::actor_error::ActorError;
  use crate::actor::actor::props::Props;
  use crate::actor::actor_system::ActorSystem;
  use crate::actor::context::ContextHandle;
  use crate::actor::context::SpawnerPart;
  use crate::actor::supervisor::SupervisorStrategyHandle;

  #[derive(Debug, Clone)]
  struct MyActor {
    is_started: Arc<AtomicBool>,
    received: Arc<Notify>,
  }

  #[async_trait]
  impl Actor for MyActor {
    async fn post_start(&self, _: ContextHandle) -> Result<(), ActorError> {
      tracing::debug!("MyActor started");
      self.is_started.store(true, Ordering::SeqCst);
      self.received.notify_one();
      Ok(())
    }

    async fn receive(&mut self, _: ContextHandle) -> Result<(), ActorError> {
      Ok(())
    }

    async fn get_supervisor_strategy(&self) -> Option<SupervisorStrategyHandle> {
      None
    }
  }

  #[tokio::test]
  async fn test_example() {
    let _ = env::set_var("RUST_LOG", "debug");
    let _ = tracing_subscriber::fmt()
      .with_env_filter(EnvFilter::from_default_env())
      .try_init();

    let system = ActorSystem::new().await.unwrap();

    let actor = MyActor {
      is_started: Arc::new(AtomicBool::new(false)),
      received: Arc::new(Notify::new()),
    };

    let actor_producer = {
      let actor = actor.clone();
      move |_| {
        let actor = actor.clone();
        async move { actor.clone() }
      }
    };

    let props = Props::from_actor_producer(actor_producer).await;

    system.get_root_context().await.spawn(props).await;

    actor.received.notified().await;

    assert_eq!(actor.is_started.load(Ordering::SeqCst), true);
  }
}