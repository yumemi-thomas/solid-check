#[cfg(feature = "node")]
pub use napi::bindgen_prelude::{Either, Error, Result};

#[cfg(not(feature = "node"))]
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(not(feature = "node"))]
#[derive(Debug)]
pub struct Error(String);

#[cfg(not(feature = "node"))]
impl Error {
    pub fn from_reason(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

#[cfg(not(feature = "node"))]
impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(not(feature = "node"))]
impl std::error::Error for Error {}

#[cfg(not(feature = "node"))]
#[derive(Clone)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
