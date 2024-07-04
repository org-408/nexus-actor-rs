use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;

use futures::future::BoxFuture;
use thiserror::Error;

use crate::actor::actor::pid::ExtendedPid;
use crate::actor::actor::props::Props;
use crate::actor::actor_system::ActorSystem;
use crate::actor::context::spawner_context_handle::SpawnerContextHandle;

#[derive(Debug, Clone, Error)]
pub enum SpawnError {
  #[error("Name already exists: {0}")]
  ErrNameExists(ExtendedPid),
}

#[derive(Clone)]
pub struct SpawnFunc(
  Arc<
    dyn Fn(ActorSystem, String, Props, SpawnerContextHandle) -> BoxFuture<'static, Result<ExtendedPid, SpawnError>>
      + Send
      + Sync,
  >,
);

impl Debug for SpawnFunc {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "SpawnFunc")
  }
}

impl PartialEq for SpawnFunc {
  fn eq(&self, _other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &_other.0)
  }
}

impl Eq for SpawnFunc {}

impl std::hash::Hash for SpawnFunc {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    (self.0.as_ref()
      as *const dyn Fn(
        ActorSystem,
        String,
        Props,
        SpawnerContextHandle,
      ) -> BoxFuture<'static, Result<ExtendedPid, SpawnError>>)
      .hash(state);
  }
}

impl SpawnFunc {
  pub fn new<F, Fut>(f: F) -> Self
  where
    F: Fn(ActorSystem, String, Props, SpawnerContextHandle) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExtendedPid, SpawnError>> + Send + 'static, {
    Self(Arc::new(move |s, name, p, sch| {
      Box::pin(f(s, name, p, sch)) as BoxFuture<'static, Result<ExtendedPid, SpawnError>>
    }))
  }

  pub async fn run(
    &self,
    actor_system: ActorSystem,
    name: &str,
    props: Props,
    parent_context: SpawnerContextHandle,
  ) -> Result<ExtendedPid, SpawnError> {
    (self.0)(actor_system, name.to_string(), props, parent_context).await
  }
}