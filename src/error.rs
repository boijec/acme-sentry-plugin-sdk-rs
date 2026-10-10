use crate::ffi::FfiString;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct PluginError {
    code: i32,
    message: String,
}

impl PluginError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        PluginError {
            code,
            message: message.into(),
        }
    }
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::new(400, msg)
    }
    pub fn cancelled() -> Self {
        Self::new(499, "operation cancelled")
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(500, msg)
    }
    pub fn code(&self) -> i32 {
        self.code
    }
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginError: {}", self.message)
    }
}

#[repr(C)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: FfiString,
}

impl From<PluginError> for ErrorInfo {
    fn from(err: PluginError) -> Self {
        ErrorInfo {
            code: err.code,
            message: FfiString::from_string(err.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostError {
    pub code: i32,
    pub message: String,
}

impl Display for HostError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl Error for HostError {}

impl ErrorInfo {
    pub(crate) fn to_host(&self) -> HostError {
        HostError {
            code: self.code,
            message: self.message.as_str().to_owned(),
        }
    }
}
