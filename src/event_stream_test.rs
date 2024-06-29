use super::*;
use crate::actor::message::{Message, MessageHandle};
use crate::event_stream::{EventStream, HandlerFunc, PredicateFunc};
use std::any::Any;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct TestString(pub String);
impl Message for TestString {
  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

#[tokio::test]
async fn test_event_stream_subscribe() {
  let es = EventStream::new();
  let s = es.subscribe(HandlerFunc::new(|_| Box::pin(async move {}))).await;
  assert!(s.is_active());
  assert_eq!(es.length(), 1);
}

#[tokio::test]
async fn test_event_stream_unsubscribe() {
  let es = EventStream::new();
  let c1 = Arc::new(AtomicI32::new(0));
  let c2 = Arc::new(AtomicI32::new(0));

  let s1 = es
    .subscribe(HandlerFunc::new({
      let c1 = Arc::clone(&c1);
      move |_| {
        let c1 = c1.clone();
        Box::pin(async move {
          c1.fetch_add(1, Ordering::SeqCst);
        })
      }
    }))
    .await;
  let s2 = es
    .subscribe(HandlerFunc::new({
      let c2 = Arc::clone(&c2);
      move |_| {
        let c2 = c2.clone();
        Box::pin(async move {
          c2.fetch_add(1, Ordering::SeqCst);
        })
      }
    }))
    .await;
  assert_eq!(es.length(), 2);

  es.unsubscribe(s2).await;
  assert_eq!(es.length(), 1);

  es.publish(MessageHandle::new(1)).await;
  assert_eq!(c1.load(Ordering::SeqCst), 1);

  es.unsubscribe(s1).await;
  assert_eq!(es.length(), 0);

  es.publish(MessageHandle::new(1)).await;
  assert_eq!(c1.load(Ordering::SeqCst), 1);
  assert_eq!(c2.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_event_stream_publish() {
  let es = EventStream::new();
  let v = Arc::new(Mutex::new(0));

  let v_clone = Arc::clone(&v);
  es.subscribe(HandlerFunc::new(move |m| {
    let v_clone = v_clone.clone();
    let m_value = if let Some(val) = m.as_any().downcast_ref::<i32>() {
      Some(*val)
    } else {
      None
    };
    Box::pin(async move {
      if let Some(val) = m_value {
        *v_clone.lock().await = val;
      }
    })
  }))
  .await;

  es.publish(MessageHandle::new(1)).await;
  assert_eq!(*v.lock().await, 1);

  es.publish(MessageHandle::new(100)).await;
  assert_eq!(*v.lock().await, 100);
}

#[tokio::test]
async fn test_event_stream_subscribe_with_predicate_is_called() {
  let es = EventStream::new();
  let called = Arc::new(Mutex::new(false));

  let called_clone = Arc::clone(&called);
  es.subscribe_with_predicate(
    HandlerFunc::new(move |_| {
      let called_clone = called_clone.clone();
      Box::pin(async move {
        *called_clone.lock().await = true;
      })
    }),
    PredicateFunc::new(|_| true),
  )
  .await;
  es.publish(MessageHandle::new(TestString("".to_string()))).await;

  assert!(*called.lock().await);
}

#[tokio::test]
async fn test_event_stream_subscribe_with_predicate_is_not_called() {
  let es = EventStream::new();
  let called = Arc::new(Mutex::new(false));

  let called_clone = Arc::clone(&called);
  es.subscribe_with_predicate(
    HandlerFunc::new(move |_| {
      let called_clone = called_clone.clone();
      Box::pin(async move {
        *called_clone.lock().await = true;
      })
    }),
    PredicateFunc::new(|_: MessageHandle| false),
  )
  .await;
  es.publish(MessageHandle::new(TestString("".to_string()))).await;

  assert!(!*called.lock().await);
}

#[derive(Debug)]
struct Event {
  i: i32,
}

impl Message for Event {
  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

#[tokio::test]
async fn test_event_stream_performance() {
  let es = EventStream::new();
  let mut subs = Vec::new();

  for i in 0..1000 {
    // Reduced iterations for faster test
    for _ in 0..10 {
      let sub = es
        .subscribe(HandlerFunc::new(move |evt| {
          let i = i; // Capture i by value
          let evt_data = if let Some(e) = evt.as_any().downcast_ref::<Event>() {
            Some(e.i)
          } else {
            None
          };
          Box::pin(async move {
            if let Some(evt_i) = evt_data {
              assert_eq!(evt_i, i, "expected i to be {} but its value is {}", i, evt_i);
            }
          })
        }))
        .await;
      subs.push(sub);
    }

    es.publish(MessageHandle::new(Event { i })).await;
    for sub in subs.drain(..) {
      es.unsubscribe(sub.clone()).await;
      assert!(!sub.is_active(), "subscription should not be active");
    }
  }
}