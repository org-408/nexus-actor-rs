#[cfg(test)]
pub mod tests {
  use crate::actor::actor::actor::Actor;
  use crate::actor::actor::actor_error::ActorError;
  use crate::actor::actor::props::Props;
  use crate::actor::actor_system::ActorSystem;
  use crate::actor::context::ContextHandle;
  use crate::actor::context::{BasePart, MessagePart, SpawnerPart, StopperPart};
  use crate::actor::message::ReceiveTimeout;
  use crate::actor::util::AsyncBarrier;
  use async_trait::async_trait;
  use std::env;
  use std::time::Duration;
  use tracing_subscriber::EnvFilter;

  #[derive(Debug, Clone)]
  struct SetReceiveTimeoutActor {
    barrier: AsyncBarrier,
  }

  impl SetReceiveTimeoutActor {
    pub fn new(barrier: AsyncBarrier) -> Self {
      Self { barrier }
    }
  }

  #[async_trait]
  impl Actor for SetReceiveTimeoutActor {
    async fn receive(&mut self, context_handle: ContextHandle) -> Result<(), ActorError> {
      let msg = context_handle.get_message_handle().await.to_typed::<ReceiveTimeout>();
      if let Some(_) = msg {
        tracing::debug!("ReceiveTimeout");
        self.barrier.wait().await;
      }
      Ok(())
    }

    async fn post_start(&self, mut ctx: ContextHandle) -> Result<(), ActorError> {
      ctx.set_receive_timeout(&Duration::from_millis(100)).await;
      Ok(())
    }
  }

  #[tokio::test]
  async fn test_example_context_set_receive_timeout() {
    let _ = env::set_var("RUST_LOG", "debug");
    let _ = tracing_subscriber::fmt()
      .with_env_filter(EnvFilter::from_default_env())
      .try_init();
    let b = AsyncBarrier::new(2);

    let system = ActorSystem::new().await.unwrap();
    let cloned_b = b.clone();

    let mut root_context = system.get_root_context().await;

    let pid = root_context
      .spawn(
        Props::from_actor_producer(move |_| {
          let cloned_b = cloned_b.clone();
          async move { SetReceiveTimeoutActor::new(cloned_b.clone()) }
        })
        .await,
      )
      .await;

    b.wait().await;

    let result = root_context.stop_future(&pid).await;

    result.result().await.unwrap();
  }
}
