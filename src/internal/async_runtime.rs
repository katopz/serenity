//! Async runtime abstraction for cross-platform support.
//!
//! This module provides a minimal abstraction layer for async runtime operations
//! that differ between native platforms and Cloudflare Workers (WASM).
//!
//! # Platform Differences
//!
//! ## Native Platforms (Linux, macOS, Windows)
//! - Full tokio runtime support
//! - Arbitrary task spawning available
//! - All time utilities available
//!
//! ## Cloudflare Workers (WASM)
//! - No tokio runtime (not compatible)
//! - No arbitrary task spawning (must complete within request cycle)
//! - Time utilities use standard library or Workers runtime
//!
//! # Usage
//!
//! ```rust,no_run
//! use serenity::internal::async_runtime::spawn_named;
//!
//! #[cfg(not(target_arch = "wasm32"))]
//! async fn background_task() {
//!     // This works on native platforms
//!     spawn_named("my_task", async move {
//!         // Do background work
//!     });
//! }
//! ```
//!
//! # Limitations in Workers
//!
//! - ❌ No arbitrary task spawning (must use inline async)
//! - ❌ No background tasks (all work within request-response cycle)
//! - ✅ Async operations work fine within request handlers

use std::future::Future;

/// Spawn a named task for debugging purposes.
///
/// On native platforms with `tokio_unstable` and `tokio_task_builder` features,
/// this will spawn a task with the given name in the format "serenity::{name}".
/// Otherwise, it spawns an unnamed task.
///
/// On WASM platforms, this function does not exist as Cloudflare Workers
/// do not support arbitrary task spawning.
///
/// # Platform Support
///
/// - ✅ Native: Spawns background task
/// - ❌ WASM: Not available (compile-time error if called)
///
/// # Example
///
/// ```rust,no_run
/// use serenity::internal::async_runtime::spawn_named;
///
/// #[cfg(not(target_arch = "wasm32"))]
/// async fn example() {
///     let handle = spawn_named("my_task", async {
///         // Background work here
///         42
///     });
///
///     let result = handle.await.unwrap();
///     assert_eq!(result, 42);
/// }
/// ```
#[cfg(feature = "http")]
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_named<F, T>(_name: &str, future: F) -> tokio::task::JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    #[cfg(all(tokio_unstable, feature = "tokio_task_builder"))]
    let handle = tokio::task::Builder::new()
        .name(&*format!("serenity::{}", _name))
        .spawn(future)
        .expect("called outside tokio runtime");

    #[cfg(not(all(tokio_unstable, feature = "tokio_task_builder")))]
    let handle = tokio::spawn(future);

    handle
}

/// Compile-time assertion that arbitrary task spawning is not supported in WASM.
///
/// Cloudflare Workers require all async work to complete within the request-response
/// cycle. Background tasks are not supported.
///
/// # Workers Pattern
///
/// Instead of spawning tasks, process all work inline:
///
/// ```rust,no_run
/// use worker::*;
///
/// #[event(fetch)]
/// async fn fetch(req: Request, env: Env) -> Result<Response> {
///     // ✅ GOOD: Inline async operations
///     let result = do_work().await?;
///
///     Response::ok(format!("Result: {}", result))
/// }
///
/// async fn do_work() -> Result<String> {
///     // All work happens here, synchronously within the request
///     Ok("done".to_string())
/// }
/// ```
///
/// # Native Pattern
///
/// On native platforms, you can spawn background tasks:
///
/// ```rust,no_run
/// use tokio::spawn;
///
/// #[tokio::main]
/// async fn main() {
///     // ✅ GOOD: Background task on native
///     spawn(async move {
///         // Background work here
///     });
/// }
/// ```
#[cfg(all(feature = "http", target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn spawn_named_is_not_supported() {
    // This function exists only to provide a compile-time error if someone tries
    // to use task spawning in WASM. The actual usage will fail to compile
    // because tokio::spawn is not available.
    compile_error!(
        "Task spawning is not supported in Cloudflare Workers. \
         All async operations must complete within the request-response cycle. \
         Process work inline instead of spawning background tasks."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "tokio_unstable")]
    #[cfg(feature = "tokio_task_builder")]
    fn test_spawn_named_is_compile_time_check() {
        // This test verifies that spawn_named compiles with task names
        let handle = spawn_named("test_task", async { 42 });
        // Don't actually run it, just verify it compiles
        drop(handle);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(all(tokio_unstable, feature = "tokio_task_builder"))]
    fn test_spawn_named_without_task_builder() {
        // This test verifies that spawn_named compiles without task names
        let handle = spawn_named("test_task", async { 42 });
        // Don't actually run it, just verify it compiles
        drop(handle);
    }
}
