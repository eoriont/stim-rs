use cxx::Exception;
use thiserror::Error;

/// Errors that can occur when interacting with Stim.
#[derive(Error, Debug)]
pub enum StimError {
    /// Error reported by the underlying Stim C++ library.
    #[error("stim error: {0}")]
    Ffi(#[from] Exception),
    /// An integer conversion failed due to platform limits.
    #[error("{0}")]
    Conversion(String),
}
