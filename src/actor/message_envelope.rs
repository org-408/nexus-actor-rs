use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::actor::message::{Message, MessageHandle};
use crate::actor::pid::ExtendedPid;

#[derive(Debug, Default, Clone)]
pub struct MessageHeaders {
  inner: HashMap<String, String>,
}

impl MessageHeaders {
  pub fn new() -> Self {
    Self { inner: HashMap::new() }
  }

  pub fn set(&mut self, key: String, value: String) {
    self.inner.insert(key, value);
  }
}

pub static EMPTY_MESSAGE_HEADER: Lazy<Arc<MessageHeaders>> = Lazy::new(|| Arc::new(MessageHeaders::new()));

pub trait ReadonlyMessageHeaders: Debug + Send + Sync + 'static {
  fn get(&self, key: &str) -> Option<&String>;
  fn keys(&self) -> Vec<&String>;
  fn length(&self) -> usize;
  fn to_map(&self) -> HashMap<String, String>;
}

#[derive(Debug, Clone)]
pub struct ReadonlyMessageHeadersHandle(Arc<dyn ReadonlyMessageHeaders>);

impl ReadonlyMessageHeadersHandle {
  pub fn new_arc(header: Arc<dyn ReadonlyMessageHeaders>) -> Self {
    ReadonlyMessageHeadersHandle(header)
  }

  pub fn new(header: impl ReadonlyMessageHeaders + Send + Sync + 'static) -> Self {
    ReadonlyMessageHeadersHandle(Arc::new(header))
  }
}

impl ReadonlyMessageHeaders for ReadonlyMessageHeadersHandle {
  fn get(&self, key: &str) -> Option<&String> {
    self.0.get(key)
  }

  fn keys(&self) -> Vec<&String> {
    self.0.keys()
  }

  fn length(&self) -> usize {
    self.0.length()
  }

  fn to_map(&self) -> HashMap<String, String> {
    self.0.to_map()
  }
}

impl ReadonlyMessageHeaders for MessageHeaders {
  fn get(&self, key: &str) -> Option<&String> {
    self.inner.get(key)
  }

  fn keys(&self) -> Vec<&String> {
    self.inner.keys().collect()
  }

  fn length(&self) -> usize {
    self.inner.len()
  }

  fn to_map(&self) -> HashMap<String, String> {
    self.inner.clone()
  }
}

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
  header: Option<MessageHeaders>,
  pub(crate) message: MessageHandle,
  sender: Option<ExtendedPid>,
}

impl Message for MessageEnvelope {
  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

impl MessageEnvelope {
  pub fn new(message: MessageHandle) -> Self {
    Self {
      header: None,
      message,
      sender: None,
    }
  }

  pub fn with_header(mut self, header: MessageHeaders) -> Self {
    self.header = Some(header);
    self
  }

  pub fn with_sender(mut self, sender: ExtendedPid) -> Self {
    self.sender = Some(sender);
    self
  }

  pub fn get_header_value(&self, key: &str) -> Option<String> {
    self.header.as_ref().and_then(|h| h.get(key).cloned())
  }

  pub fn set_header(&mut self, key: String, value: String) {
    if self.header.is_none() {
      self.header = Some(MessageHeaders::default());
    }
    if let Some(h) = &mut self.header {
      h.set(key, value);
    }
  }

  pub fn get_headers(&self) -> Option<MessageHeaders> {
    self.header.clone()
  }
}

pub fn wrap_envelope(message: MessageHandle) -> Arc<MessageEnvelope> {
  if let Some(envelope) = message.as_any().downcast_ref::<MessageEnvelope>() {
    Arc::new(envelope.clone())
  } else {
    Arc::new(MessageEnvelope::new(message))
  }
}

pub fn unwrap_envelope(message: MessageHandle) -> (Option<MessageHeaders>, MessageHandle, Option<ExtendedPid>) {
  if let Some(envelope) = message.as_any().downcast_ref::<MessageEnvelope>() {
    (
      envelope.header.clone(),
      envelope.message.clone(),
      envelope.sender.clone(),
    )
  } else {
    (None, message, None)
  }
}

pub fn unwrap_envelope_header(message: MessageHandle) -> Option<MessageHeaders> {
  if let Some(envelope) = message.as_any().downcast_ref::<MessageEnvelope>() {
    envelope.header.clone()
  } else {
    None
  }
}

pub fn unwrap_envelope_message(message: MessageHandle) -> MessageHandle {
  if let Some(envelope) = message.as_any().downcast_ref::<MessageEnvelope>() {
    envelope.message.clone()
  } else {
    message
  }
}

pub fn unwrap_envelope_sender(message: MessageHandle) -> Option<ExtendedPid> {
  if let Some(envelope) = message.as_any().downcast_ref::<MessageEnvelope>() {
    envelope.sender.clone()
  } else {
    None
  }
}

#[derive(Debug, Clone)]
pub struct MessageOrEnvelope {
  message: Option<MessageHandle>,
  envelope: Option<MessageEnvelope>,
  sender: Option<ExtendedPid>,
}

impl Message for MessageOrEnvelope {
  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

impl MessageOrEnvelope {
  pub fn of_message(message: MessageHandle) -> Self {
    Self {
      message: Some(message),
      envelope: None,
      sender: None,
    }
  }

  pub fn of_envelope(envelope: MessageEnvelope) -> Self {
    Self {
      message: None,
      envelope: Some(envelope),
      sender: None,
    }
  }

  pub fn with_sender(mut self, sender: Option<ExtendedPid>) -> Self {
    self.sender = sender;
    self
  }

  pub fn get_value(&self) -> MessageHandle {
    match (self.message.clone(), self.envelope.clone()) {
      (Some(msg), _) => msg,
      (_, Some(env)) => env.message.clone(),
      _ => panic!("MessageOrEnvelope is empty"),
    }
  }

  pub fn get_message(&self) -> Option<MessageHandle> {
    self.message.clone()
  }

  pub fn get_envelope(&self) -> Option<MessageEnvelope> {
    self.envelope.clone()
  }

  pub fn get_sender(&self) -> Option<ExtendedPid> {
    self.sender.clone()
  }
}
