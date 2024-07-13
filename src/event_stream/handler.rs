use crate::actor::message::message_handle::MessageHandle;
use futures::future::BoxFuture;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;

// Handler defines a callback function that must be passed when subscribing.
#[derive(Clone)]
pub struct Handler(Arc<dyn Fn(MessageHandle) -> BoxFuture<'static, ()> + Send + Sync + 'static>);

impl Handler {
  pub fn new<F, Fut>(f: F) -> Self
  where
    F: Fn(MessageHandle) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static, {
    Self(Arc::new(move |mh| Box::pin(f(mh)) as BoxFuture<'static, ()>))
  }

  pub async fn run(&self, evt: MessageHandle) {
    (self.0)(evt).await
  }
}

impl Debug for Handler {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Handler")
  }
}

impl PartialEq for Handler {
  fn eq(&self, _other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &_other.0)
  }
}

impl Eq for Handler {}

impl std::hash::Hash for Handler {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    (self.0.as_ref() as *const dyn Fn(MessageHandle) -> BoxFuture<'static, ()>).hash(state);
  }
}
