use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;

use futures::future::BoxFuture;

use crate::actor::actor::pid::ExtendedPid;
use crate::actor::context::sender_context_handle::SenderContextHandle;
use crate::actor::message::message_envelope::MessageEnvelope;

// SenderFunc
#[derive(Clone)]
pub struct SenderMiddlewareChainFunc(
  Arc<dyn Fn(SenderContextHandle, ExtendedPid, MessageEnvelope) -> BoxFuture<'static, ()> + Send + Sync>,
);

impl Debug for SenderMiddlewareChainFunc {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "SenderFunc")
  }
}

impl PartialEq for SenderMiddlewareChainFunc {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0)
  }
}

impl Eq for SenderMiddlewareChainFunc {}

impl std::hash::Hash for SenderMiddlewareChainFunc {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    (self.0.as_ref() as *const dyn Fn(SenderContextHandle, ExtendedPid, MessageEnvelope) -> BoxFuture<'static, ()>)
      .hash(state);
  }
}

impl SenderMiddlewareChainFunc {
  pub fn new<F, Fut>(f: F) -> Self
  where
    F: Fn(SenderContextHandle, ExtendedPid, MessageEnvelope) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static, {
    Self(Arc::new(move |sch, ep, me| {
      Box::pin(f(sch, ep, me)) as BoxFuture<'static, ()>
    }))
  }

  pub async fn run(&self, context: SenderContextHandle, target: ExtendedPid, envelope: MessageEnvelope) {
    (self.0)(context, target, envelope).await;
  }
}