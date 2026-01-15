//! Synchronization primitives abstraction for cross-platform support

#[cfg(feature = "wasm")]
pub use parking_lot::Mutex;

#[cfg(feature = "wasm")]
pub use parking_lot::RwLock;

#[cfg(feature = "wasm")]
pub use parking_lot::OnceLock;

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::Mutex;

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::RwLock;

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::OnceLock;

#[cfg(feature = "wasm")]
pub use futures::channel::oneshot::{self, Receiver, Sender};

#[cfg(not(feature = "wasm"))]
pub use tokio::sync::oneshot::{self, Receiver, Sender};
