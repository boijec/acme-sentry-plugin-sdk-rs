use crate::ffi::{FfiString, FfiVec};

#[repr(C)]
pub struct KVPair {
    pub key: FfiString,
    pub value: Value,
}

#[repr(C, u8)]
pub enum Value {
    Unit,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(FfiString),
    List(FfiVec<Value>),
    Map(FfiVec<KVPair>),
}

impl Value {
    pub fn str(s: impl Into<String>) -> Self {
        Value::Str(FfiString::from_string(s.into()))
    }
    pub fn list(items: Vec<Value>) -> Self {
        Value::List(FfiVec::from_vec(items))
    }
    pub fn map(pairs: Vec<(&str, Value)>) -> Self {
        let pairs = pairs
            .into_iter()
            .map(|(k, v)| KVPair {
                key: FfiString::from_string(k.to_owned()),
                value: v,
            })
            .collect();
        Value::Map(FfiVec::from_vec(pairs))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(str) => Some(str.as_str()),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(v) => Some(v.as_slice()),
            _ => None,
        }
    }
    pub fn as_map(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(pairs) => pairs
                .as_slice()
                .iter()
                .find(|kv| kv.key.as_str() == key)
                .map(|kv| &kv.value),
            _ => None,
        }
    }
}
