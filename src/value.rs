use crate::ffi::{FfiString, FfiVec};

#[repr(C)]
pub struct KVPair {
    pub key: FfiString,
    pub value: PluginValue,
}

#[repr(C, u8)]
pub enum PluginValue {
    Unit,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(FfiString),
    List(FfiVec<PluginValue>),
    Map(FfiVec<KVPair>),
}

#[repr(C, u8)]
pub enum HostValue {
    Unit,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    List(Vec<HostValue>),
    Map(Vec<(String, HostValue)>),
}

impl PluginValue {
    pub fn str(s: impl Into<String>) -> Self {
        PluginValue::Str(FfiString::from_string(s.into()))
    }
    pub fn list(items: Vec<PluginValue>) -> Self {
        PluginValue::List(FfiVec::from_vec(items))
    }
    pub fn map(pairs: Vec<(&str, PluginValue)>) -> Self {
        let pairs = pairs
            .into_iter()
            .map(|(k, v)| KVPair {
                key: FfiString::from_string(k.to_owned()),
                value: v,
            })
            .collect();
        PluginValue::Map(FfiVec::from_vec(pairs))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PluginValue::Str(str) => Some(str.as_str()),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PluginValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PluginValue::I64(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PluginValue::F64(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&[PluginValue]> {
        match self {
            PluginValue::List(v) => Some(v.as_slice()),
            _ => None,
        }
    }
    pub fn as_map(&self, key: &str) -> Option<&PluginValue> {
        match self {
            PluginValue::Map(pairs) => pairs
                .as_slice()
                .iter()
                .find(|kv| kv.key.as_str() == key)
                .map(|kv| &kv.value),
            _ => None,
        }
    }
    pub(crate) fn to_host(&self) -> HostValue {
        match self {
            PluginValue::Unit => HostValue::Unit,
            PluginValue::Bool(b) => HostValue::Bool(*b),
            PluginValue::I64(n) => HostValue::I64(*n),
            PluginValue::F64(n) => HostValue::F64(*n),
            PluginValue::Str(s) => HostValue::Str(s.as_str().to_owned()),
            PluginValue::List(v) => {
                HostValue::List(v.as_slice().iter().map(PluginValue::to_host).collect())
            }
            PluginValue::Map(p) => HostValue::Map(
                p.as_slice()
                    .iter()
                    .map(|kv| (kv.key.as_str().to_owned(), kv.value.to_host()))
                    .collect(),
            ),
        }
    }
}

impl HostValue {
    pub fn str(s: impl Into<String>) -> Self {
        HostValue::Str(s.into())
    }
    pub fn list(items: Vec<HostValue>) -> Self {
        HostValue::List(items)
    }
    pub fn map(pairs: Vec<(&str, HostValue)>) -> Self {
        HostValue::Map(pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            HostValue::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            HostValue::I64(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            HostValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn get(&self, key: &str) -> Option<&HostValue> {
        match self {
            HostValue::Map(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub(crate) fn to_ffi(&self) -> PluginValue {
        match self {
            HostValue::Unit => PluginValue::Unit,
            HostValue::Bool(b) => PluginValue::Bool(*b),
            HostValue::I64(n) => PluginValue::I64(*n),
            HostValue::F64(n) => PluginValue::F64(*n),
            HostValue::Str(s) => PluginValue::Str(FfiString::from_string(s.clone())),
            HostValue::List(items) => {
                PluginValue::list(items.iter().map(HostValue::to_ffi).collect())
            }
            HostValue::Map(pairs) => PluginValue::Map(FfiVec::from_vec(
                pairs
                    .iter()
                    .map(|(k, v)| KVPair {
                        key: FfiString::from_string(k.clone()),
                        value: v.to_ffi(),
                    })
                    .collect(),
            )),
        }
    }
}
