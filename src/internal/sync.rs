//! Synchronization primitives abstraction for cross-platform support

#[cfg(feature = "wasm")]
pub use parking_lot::Mutex;

#[cfg(feature = "wasm")]
pub use parking_lot::RwLock;

// OnceLock is available in std since Rust 1.70, and our rust-version is 1.74
// Use std::sync::OnceLock for both platforms for better compatibility
pub use std::sync::OnceLock;

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::Mutex;

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::RwLock;

#[cfg(feature = "wasm")]
pub use futures::channel::oneshot::{self, Receiver, Sender};

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::oneshot::{self, Receiver, Sender};
