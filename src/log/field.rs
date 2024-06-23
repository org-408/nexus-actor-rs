use std::any::Any;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::log::encoder::Encoder;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
  Unknown,
  Bool,
  Float,
  Int,
  Int64,
  Duration,
  Uint,
  Uint64,
  String,
  Stringer,
  Error,
  Object,
  TypeOf,
  Skip,
  Caller,
}

#[derive(Debug, Clone)]
pub struct Field {
  key: String,
  field_type: FieldType,
  val: i64,
  str: String,
  obj: Option<Arc<dyn Any + Send + Sync>>,
}

impl PartialEq for Field {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key && self.field_type == other.field_type && self.val == other.val && self.str == other.str
  }
}

impl Field {
  pub fn bool(key: &str, val: bool) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Bool,
      val: if val { 1 } else { 0 },
      str: String::new(),
      obj: None,
    }
  }

  pub fn float64(key: &str, val: f64) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Float,
      val: val.to_bits() as i64,
      str: String::new(),
      obj: None,
    }
  }

  pub fn int(key: &str, val: i32) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Int,
      val: val as i64,
      str: String::new(),
      obj: None,
    }
  }

  pub fn int64(key: &str, val: i64) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Int64,
      val,
      str: String::new(),
      obj: None,
    }
  }

  pub fn uint(key: &str, val: u32) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Uint,
      val: val as i64,
      str: String::new(),
      obj: None,
    }
  }

  pub fn uint64(key: &str, val: u64) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Uint64,
      val: val as i64,
      str: String::new(),
      obj: None,
    }
  }

  pub fn string(key: &str, val: &str) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::String,
      val: 0,
      str: val.to_string(),
      obj: None,
    }
  }

  pub fn stringer<T: fmt::Display + Send + Sync + 'static>(key: &str, val: T) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Stringer,
      val: 0,
      str: String::new(),
      obj: Some(Arc::new(val)),
    }
  }

  pub fn time(key: &str, val: SystemTime) -> Self {
    let duration = val.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = duration.as_secs_f64();
    Self::float64(key, seconds)
  }

  pub fn error(err: &dyn Error) -> Self {
    Field {
      key: "error".to_string(),
      field_type: FieldType::Error,
      val: 0,
      str: String::new(),
      obj: Some(Arc::new(err.to_string())),
    }
  }

  // Stack関数の実装はRustでは複雑になるため、別途検討が必要です。

  pub fn duration(key: &str, val: Duration) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Duration,
      val: val.as_nanos() as i64,
      str: String::new(),
      obj: None,
    }
  }

  pub fn object<T: Send + Sync + 'static>(key: &str, val: T) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::Object,
      val: 0,
      str: String::new(),
      obj: Some(Arc::new(val)),
    }
  }

  pub fn type_of<T: 'static>(key: &str, _: &T) -> Self {
    Field {
      key: key.to_string(),
      field_type: FieldType::TypeOf,
      val: 0,
      str: String::new(),
      obj: Some(Arc::new(std::any::TypeId::of::<T>())),
    }
  }

  pub fn message<T: Send + Sync + 'static>(val: T) -> Self {
    Self::object("message", val)
  }

  // CallerSkip と Caller の実装はRustでは異なるアプローチが必要です。
  // 例えば、backtrace クレートを使用することができます。

  pub fn encode(&self, enc: &mut dyn Encoder) {
    match self.field_type {
      FieldType::Bool => enc.encode_bool(&self.key, self.val != 0),
      FieldType::Float => enc.encode_float64(&self.key, f64::from_bits(self.val as u64)),
      FieldType::Int => enc.encode_int(&self.key, self.val as i32),
      FieldType::Int64 => enc.encode_int64(&self.key, self.val),
      FieldType::Duration => enc.encode_duration(&self.key, Duration::from_nanos(self.val as u64)),
      FieldType::Uint => enc.encode_uint(&self.key, self.val as u32),
      FieldType::Uint64 => enc.encode_uint64(&self.key, self.val as u64),
      FieldType::String => enc.encode_string(&self.key, &self.str),
      FieldType::Stringer => {
        if let Some(obj) = &self.obj {
          if let Some(stringer) = obj.downcast_ref::<Box<dyn fmt::Display>>() {
            enc.encode_string(&self.key, &stringer.to_string());
          }
        }
      }
      FieldType::Error => {
        if let Some(obj) = &self.obj {
          if let Some(err_str) = obj.downcast_ref::<String>() {
            enc.encode_string(&self.key, err_str);
          }
        }
      }
      FieldType::Object => {
        if let Some(obj) = &self.obj {
          enc.encode_object(&self.key, obj.as_ref());
        }
      }
      FieldType::TypeOf => {
        if let Some(obj) = &self.obj {
          if let Some(type_id) = obj.downcast_ref::<std::any::TypeId>() {
            enc.encode_type(&self.key, *type_id);
          }
        }
      }
      FieldType::Caller => {
        // CallerInfo の実装が必要です
      }
      FieldType::Skip => {}
      FieldType::Unknown => panic!("unknown field type found"),
    }
  }
}
