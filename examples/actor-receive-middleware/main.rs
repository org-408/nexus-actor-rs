use nexus_acto_rs::actor::actor::{Logger, Props};
use nexus_acto_rs::actor::actor_system::ActorSystem;
use nexus_acto_rs::actor::context::{MessagePart, SenderPart, SpawnerPart};
use nexus_acto_rs::actor::message::{Message, MessageHandle};
use nexus_acto_rs::Message;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, PartialEq, Eq, Message)]
struct Hello {
  who: String,
}

#[tokio::main]
async fn main() {
  let _ = env::set_var("RUST_LOG", "debug");
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

  let system = ActorSystem::new().await;
  let mut root_context = system.get_root_context().await;
  let props = Props::from_actor_receiver_with_opts(
    |ctx| async move {
      let message_handle_opt = ctx.get_message_handle_opt().await;
      if let Some(message_handle) = message_handle_opt {
        tracing::debug!("Message handle: {:?}", message_handle);
        let msg_opt = message_handle.to_typed::<Hello>();
        tracing::debug!("Message: {:?}", msg_opt);
      }
      Ok(())
    },
    [Props::with_receiver_middlewares([Logger::of_receiver()])],
  )
  .await;
  let pid = root_context.spawn(props).await;
  let msg = MessageHandle::new(Hello {
    who: "world".to_string(),
  });
  root_context.send(pid, msg).await;
  sleep(Duration::from_secs(5)).await;
}
