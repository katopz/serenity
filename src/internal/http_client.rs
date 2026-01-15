//! HTTP client abstraction for cross-platform support

#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::{Client, ClientBuilder};

#[cfg(target_arch = "wasm32")]
pub use reqwest_wasm::{Client, ClientBuilder};

// Common types from the http crate - both reqwest and reqwest-wasm use this
pub use http::HeaderMap;
pub use http::HeaderValue;
pub use http::Method;
pub use http::StatusCode;
pub use http::Uri;

// Re-export Url from url crate
pub use url::Url;

// IntoUrl trait for native (reqwest provides this)
#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::IntoUrl;

// Implement IntoUrl trait for WASM since reqwest-wasm doesn't provide it
#[cfg(target_arch = "wasm32")]
pub trait IntoUrl: Send + Sync {
    fn into_url(self) -> Result<Url, url::ParseError>;
}

#[cfg(target_arch = "wasm32")]
impl IntoUrl for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(&self)
    }
}

#[cfg(target_arch = "wasm32")]
impl IntoUrl for &str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self)
    }
}

#[cfg(target_arch = "wasm32")]
impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }
}

// Re-export error types for convenience
#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::Error as HttpClientError;

#[cfg(target_arch = "wasm32")]
pub use reqwest_wasm::Error as HttpClientError;

// Response type - both use the http::Response wrapper
#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::Response;

#[cfg(target_arch = "wasm32")]
pub use reqwest_wasm::Response;
