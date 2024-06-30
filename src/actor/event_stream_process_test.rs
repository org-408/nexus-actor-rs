use std::any::Any;

use tokio::sync::mpsc;

use crate::actor::actor_system::ActorSystem;
use crate::actor::context::SenderPart;
use crate::actor::message::{Message, MessageHandle};
use crate::event_stream::HandlerFunc;

#[derive(Debug, Clone)]
struct EsTestMsg;

impl Message for EsTestMsg {
  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

#[tokio::test]
async fn test_sends_messages_to_event_stream() {
  let test_cases = vec![
    ("plain", MessageHandle::new(EsTestMsg)),
    ("envelope", MessageHandle::new(EsTestMsg)),
  ];

  for (_, message) in test_cases {
    let system = ActorSystem::new(&[]).await;
    let (tx, mut rx) = mpsc::channel(5);
    let event_stream = system.get_event_stream().await;

    let subscription = event_stream
      .subscribe(HandlerFunc::new(move |evt| {
        let cloned_tx = tx.clone();
        async move {
          if evt.as_any().is::<EsTestMsg>() {
            cloned_tx.send(()).await.unwrap();
          }
        }
      }))
      .await;

    let pid = system.new_local_pid("eventstream").await;

    system.get_root_context().await.send(pid, message).await;

    rx.recv().await.unwrap();

    event_stream.unsubscribe(subscription).await;
  }
}