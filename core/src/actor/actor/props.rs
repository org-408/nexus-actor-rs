use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;
use tokio::sync::Mutex;

use crate::actor::actor::actor::Actor;
use crate::actor::actor::actor_error::ActorError;
use crate::actor::actor::actor_handle::ActorHandle;
use crate::actor::actor::actor_process::ActorProcess;
use crate::actor::actor::actor_producer::ActorProducer;
use crate::actor::actor::actor_receiver::ActorReceiver;
use crate::actor::actor::context_decorator::ContextDecorator;
use crate::actor::actor::context_decorator_chain::ContextDecoratorChain;
use crate::actor::actor::context_handler::ContextHandler;
use crate::actor::actor::middleware_chain::{
  make_context_decorator_chain, make_receiver_middleware_chain, make_sender_middleware_chain,
  make_spawn_middleware_chain,
};
use crate::actor::actor::pid::ExtendedPid;
use crate::actor::actor::receiver_middleware::ReceiverMiddleware;
use crate::actor::actor::receiver_middleware_chain::ReceiverMiddlewareChain;
use crate::actor::actor::sender_middleware::SenderMiddleware;
use crate::actor::actor::sender_middleware_chain::SenderMiddlewareChain;
use crate::actor::actor::spawn_middleware::SpawnMiddleware;
use crate::actor::actor::spawner::{SpawnError, Spawner};
use crate::actor::actor_system::ActorSystem;
use crate::actor::context::ActorContext;
use crate::actor::context::ContextHandle;
use crate::actor::context::SpawnerContextHandle;
use crate::actor::context::{InfoPart, ReceiverPart};
use crate::actor::dispatch::unbounded_mailbox_creator_with_opts;
use crate::actor::dispatch::Mailbox;
use crate::actor::dispatch::MailboxHandle;
use crate::actor::dispatch::MailboxProducer;
use crate::actor::dispatch::*;
use crate::actor::message::AutoReceiveMessage;
use crate::actor::message::MessageHandle;
use crate::actor::message::SystemMessage;
use crate::actor::metrics::metrics::{Metrics, EXTENSION_ID};
use crate::actor::process::ProcessHandle;
use crate::actor::supervisor::SupervisorStrategyHandle;
use crate::actor::supervisor::DEFAULT_SUPERVISION_STRATEGY;
use crate::metrics::LIB_NAME;

#[derive(Debug, Clone)]
pub struct Props {
  spawner: Option<Spawner>,
  pub(crate) producer: Option<ActorProducer>,
  mailbox_producer: Option<MailboxProducer>,
  guardian_strategy: Option<SupervisorStrategyHandle>,
  supervisor_strategy: Option<SupervisorStrategyHandle>,
  dispatcher: Option<DispatcherHandle>,
  receiver_middleware: Vec<ReceiverMiddleware>,
  sender_middleware: Vec<SenderMiddleware>,
  spawn_middleware: Vec<SpawnMiddleware>,
  receiver_middleware_chain: Option<ReceiverMiddlewareChain>,
  sender_middleware_chain: Option<SenderMiddlewareChain>,
  spawn_middleware_chain: Option<Spawner>,
  context_decorator: Vec<ContextDecorator>,
  context_decorator_chain: Option<ContextDecoratorChain>,
  on_init: Vec<ContextHandler>,
}

static_assertions::assert_impl_all!(Props: Send, Sync);

static DEFAULT_DISPATCHER: Lazy<DispatcherHandle> =
  Lazy::new(|| DispatcherHandle::new(TokioRuntimeContextDispatcher::new().unwrap()));
static DEFAULT_MAILBOX_PRODUCER: Lazy<MailboxProducer> = Lazy::new(|| unbounded_mailbox_creator_with_opts(vec![]));

static DEFAULT_SPAWNER: Lazy<Spawner> = Lazy::new(|| {
  Spawner::new(
    |actor_system: ActorSystem, name: String, props: Props, parent_context: SpawnerContextHandle| {
      async move {
        tracing::debug!("Spawn actor: {}", name);
        let mut ctx = ActorContext::new(actor_system.clone(), props.clone(), parent_context.get_self_opt().await).await;
        let mut mb = props.produce_mailbox().await;
        // prepare the mailbox number counter

        if let Some(mp) = actor_system.get_config().await.metrics_provider {
          if let Some(extension_arc) = actor_system.get_extensions().await.get(*EXTENSION_ID).await {
            let mut extension_mg = extension_arc.lock().await;
            if let Some(metrics) = extension_mg.as_any_mut().downcast_mut::<Metrics>() {
              if metrics.enabled() {
                let actor_mailbox_length_observable_gauge = metrics
                  .get_metrics()
                  .unwrap()
                  .instruments()
                  .actor_mailbox_length_observable_gauge();
                metrics.prepare_mailbox_length_gauge();
                let len = mb.get_user_messages_count().await;
                let meter = mp.meter(LIB_NAME);
                let result = meter.register_callback(&[], move |observer| {
                  observer.observe_i64(&actor_mailbox_length_observable_gauge, len as i64, &[]);
                });
                if let Err(e) = result {
                  tracing::error!("Failed to register mailbox length callback: {:?}", e);
                }
              }
            }
          }
        }

        let dp = props.get_dispatcher();
        let proc = ActorProcess::new(mb.clone());
        let proc_handle = ProcessHandle::new(proc);
        let pr = actor_system.get_process_registry().await;

        let (pid, absent) = pr.add_process(proc_handle, &name);
        if !absent {
          return Err(SpawnError::ErrNameExists(pid.clone()));
        }

        ctx.set_self(pid.clone()).await;

        initialize(props, ctx.clone());

        let mut mi = MessageInvokerHandle::new(Arc::new(Mutex::new(ctx.clone())));

        mb.register_handlers(Some(mi.clone()), Some(dp.clone())).await;
        tracing::debug!("mailbox handlers registered: {}", name);

        let result = mi
          .invoke_user_message(MessageHandle::new(AutoReceiveMessage::PreStart))
          .await;

        if result.is_err() {
          return Err(SpawnError::ErrPreStart(result.err().unwrap()));
        }

        mb.post_system_message(MessageHandle::new(SystemMessage::Start)).await;
        tracing::debug!("post_system_message: started: {}", name);
        mb.start().await;
        tracing::debug!("mailbox started: {}", name);

        Ok(pid)
      }
    },
  )
});

fn initialize(props: Props, ctx: ActorContext) {
  for init in props.on_init {
    init.run(ContextHandle::new(ctx.clone()));
  }
}

#[derive(Debug, Clone)]
pub struct ActorReceiverActor(ActorReceiver);

impl ActorReceiverActor {
  pub fn new(actor_receiver: ActorReceiver) -> Self {
    Self(actor_receiver)
  }
}

#[async_trait]
impl Actor for ActorReceiverActor {
  async fn handle(&mut self, ctx: ContextHandle) -> Result<(), ActorError> {
    self.0.run(ctx).await
  }

  async fn receive(&mut self, _: ContextHandle) -> Result<(), ActorError> {
    Ok(())
  }

  async fn get_supervisor_strategy(&self) -> Option<SupervisorStrategyHandle> {
    None
  }
}

#[derive(Clone)]
pub struct PropsOption(Arc<Mutex<dyn FnMut(&mut Props) + Send + Sync + 'static>>);

impl PropsOption {
  pub fn new(f: impl FnMut(&mut Props) + Send + Sync + 'static) -> Self {
    Self(Arc::new(Mutex::new(f)))
  }

  pub async fn run(&self, props: &mut Props) {
    let mut mg = self.0.lock().await;
    mg(props)
  }
}

impl Props {
  pub fn with_on_init(mut init: Vec<ContextHandler>) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.on_init.append(&mut init);
    })
  }

  pub fn with_actor_producer(producer: ActorProducer) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.producer = Some(producer.clone());
    })
  }

  pub fn with_actor_receiver(actor_receiver: ActorReceiver) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      let actor_receiver = actor_receiver.clone();
      props.producer = Some(ActorProducer::from_handle(move |_| {
        let actor_receiver = actor_receiver.clone();
        async move {
          let actor = ActorReceiverActor(actor_receiver.clone());
          ActorHandle::new(actor)
        }
      }));
    })
  }

  pub fn with_dispatcher(dispatcher: DispatcherHandle) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.dispatcher = Some(dispatcher.clone());
    })
  }

  pub fn with_mailbox_producer(mailbox_producer: MailboxProducer) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.mailbox_producer = Some(mailbox_producer.clone());
    })
  }

  pub fn with_context_decorators(decorators: impl IntoIterator<Item = ContextDecorator> + Send + Sync) -> PropsOption {
    let cloned_decorators = decorators.into_iter().collect::<Vec<_>>();
    PropsOption::new(move |props: &mut Props| {
      let cloned_decorators = cloned_decorators.clone();
      props.context_decorator.extend(cloned_decorators.clone());
      props.context_decorator_chain = make_context_decorator_chain(
        &props.context_decorator,
        ContextDecoratorChain::new(move |ch| {
          let cloned_ch = ch.clone();
          async move { cloned_ch.clone() }
        }),
      );
    })
  }

  pub fn with_guardian(guardian: SupervisorStrategyHandle) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.guardian_strategy = Some(guardian.clone());
    })
  }

  pub fn with_supervisor_strategy(supervisor: SupervisorStrategyHandle) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.supervisor_strategy = Some(supervisor.clone());
    })
  }

  pub fn with_receiver_middlewares(
    middlewares: impl IntoIterator<Item = ReceiverMiddleware> + Send + Sync,
  ) -> PropsOption {
    let middlewares = middlewares.into_iter().collect::<Vec<_>>();
    PropsOption::new(move |props: &mut Props| {
      props.receiver_middleware.extend(middlewares.clone());
      props.receiver_middleware_chain = make_receiver_middleware_chain(
        &props.receiver_middleware,
        ReceiverMiddlewareChain::new(|mut rch, me| async move { rch.receive(me).await }),
      );
    })
  }

  pub fn with_sender_middlewares(middlewares: impl IntoIterator<Item = SenderMiddleware> + Send + Sync) -> PropsOption {
    let middlewares = middlewares.into_iter().collect::<Vec<_>>();
    PropsOption::new(move |props: &mut Props| {
      props.sender_middleware.extend(middlewares.clone());
      props.sender_middleware_chain = make_sender_middleware_chain(
        &props.sender_middleware,
        SenderMiddlewareChain::new(|sch, target, me| async move {
          target
            .send_user_message(sch.get_actor_system().await.clone(), MessageHandle::new(me))
            .await
        }),
      );
    })
  }

  pub fn with_spawner(spawner: Spawner) -> PropsOption {
    PropsOption::new(move |props: &mut Props| {
      props.spawner = Some(spawner.clone());
    })
  }

  pub fn with_spawn_middleware(
    spawn_middlewares: impl IntoIterator<Item = SpawnMiddleware> + Send + Sync,
  ) -> PropsOption {
    let spawn_middlewares = spawn_middlewares.into_iter().collect::<Vec<_>>();
    PropsOption::new(move |props: &mut Props| {
      props.spawn_middleware.extend(spawn_middlewares.clone());
      props.spawn_middleware_chain = make_spawn_middleware_chain(
        &props.spawn_middleware,
        Spawner::new(move |s, id, p, sch| async move {
          if let Some(spawner) = &p.spawner {
            spawner.run(s, &id, p.clone(), sch).await
          } else {
            DEFAULT_SPAWNER.run(s, &id, p, sch).await
          }
        }),
      );
    })
  }

  fn get_spawner(&self) -> Spawner {
    self.spawner.clone().unwrap_or(DEFAULT_SPAWNER.clone())
  }

  pub(crate) fn get_producer(&self) -> ActorProducer {
    self.producer.clone().unwrap()
  }

  fn get_dispatcher(&self) -> DispatcherHandle {
    self.dispatcher.clone().unwrap_or_else(|| DEFAULT_DISPATCHER.clone())
  }

  pub(crate) fn get_supervisor_strategy(&self) -> SupervisorStrategyHandle {
    self
      .supervisor_strategy
      .clone()
      .unwrap_or_else(|| DEFAULT_SUPERVISION_STRATEGY.clone())
  }

  pub(crate) fn get_spawn_middleware_chain(&self) -> Option<Spawner> {
    self.spawn_middleware_chain.clone()
  }

  pub(crate) fn get_guardian_strategy(&self) -> Option<SupervisorStrategyHandle> {
    self.guardian_strategy.clone()
  }

  pub(crate) fn get_sender_middleware_chain(&self) -> Option<SenderMiddlewareChain> {
    self.sender_middleware_chain.clone()
  }

  pub(crate) fn get_receiver_middleware_chain(&self) -> Option<ReceiverMiddlewareChain> {
    self.receiver_middleware_chain.clone()
  }

  pub(crate) fn get_context_decorator_chain(&self) -> Option<ContextDecoratorChain> {
    self.context_decorator_chain.clone()
  }

  async fn produce_mailbox(&self) -> MailboxHandle {
    if let Some(mailbox_producer) = &self.mailbox_producer {
      mailbox_producer.run().await
    } else {
      DEFAULT_MAILBOX_PRODUCER.run().await
    }
  }

  pub async fn from_actor_producer<A, F, Fut>(f: F) -> Props
  where
    A: Actor,
    F: Fn(ContextHandle) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = A> + Send + 'static, {
    Props::from_actor_producer_with_opts(f, []).await
  }

  pub async fn from_actor_producer_with_opts<A, F, Fut>(f: F, opts: impl IntoIterator<Item = PropsOption>) -> Props
  where
    A: Actor,
    F: Fn(ContextHandle) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = A> + Send + 'static, {
    let producer = ActorProducer::new(f);
    let opts = opts.into_iter().collect::<Vec<_>>();
    let mut props = Props {
      on_init: vec![],
      producer: Some(producer),
      dispatcher: None,
      mailbox_producer: None,
      context_decorator: vec![],
      guardian_strategy: None,
      supervisor_strategy: None,
      receiver_middleware: vec![],
      sender_middleware: vec![],
      spawner: None,
      spawn_middleware: vec![],
      receiver_middleware_chain: None,
      sender_middleware_chain: None,
      spawn_middleware_chain: None,
      context_decorator_chain: None,
    };
    props.configure(&opts).await;
    props
  }

  pub async fn from_actor_receiver_with_opts<F, Fut>(f: F, opts: impl IntoIterator<Item = PropsOption>) -> Props
  where
    F: Fn(ContextHandle) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), ActorError>> + Send + 'static, {
    let actor_receiver = ActorReceiver::new(f);
    let opts = opts.into_iter().collect::<Vec<_>>();
    let producer = move |_| {
      let cloned = actor_receiver.clone();
      async move {
        let actor = ActorReceiverActor(cloned);
        ActorHandle::new(actor)
      }
    };
    Props::from_actor_producer_with_opts(producer, opts).await
  }

  pub async fn from_actor_receiver<F, Fut>(f: F) -> Props
  where
    F: Fn(ContextHandle) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), ActorError>> + Send + 'static, {
    Self::from_actor_receiver_with_opts(f, []).await
  }

  pub(crate) async fn spawn(
    self,
    actor_system: ActorSystem,
    name: &str,
    parent_context: SpawnerContextHandle,
  ) -> Result<ExtendedPid, SpawnError> {
    self.get_spawner().run(actor_system, name, self, parent_context).await
  }

  async fn configure(&mut self, opts: &[PropsOption]) -> &mut Self {
    for opt in opts {
      opt.run(self).await;
    }
    self
  }
}