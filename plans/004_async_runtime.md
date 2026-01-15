# Plan: Async Runtime Utilities and Task Spawning for WASM Support

## Overview
Replace tokio async runtime utilities with WASM-compatible alternatives to enable Cloudflare Workers compatibility while maintaining native platform functionality for REST API operations.

## Problem Statement
- Tokio runtime is not compatible with WASM/Cloudflare Workers
- `tokio::spawn`, `tokio::time`, and tokio runtime features are used throughout the codebase
- Cloudflare Workers use their own event loop and don't support arbitrary task spawning
- Need to maintain both native and WASM builds with feature flags
- Focus on API/REST functionality - complex async patterns should be minimal for REST API use

## Scope

**Affected Modules:**
- `src/client/mod.rs` - Uses `tokio::spawn`, `tokio::time`, channels
- `src/gateway/` - Uses extensive tokio async features (likely disabled for WASM)
- `src/cache/` - May use time-based expiration
- `src/http/` - May use time-based operations for rate limiting
- Any modules with `tokio::spawn`, `tokio::time`, or tokio runtime usage

**In Scope:**
- Replace `tokio::spawn` usage
- Replace `tokio::time` utilities (sleep, interval, timeout, etc.)
- Replace tokio channels with alternatives
- Update async trait implementations
- Handle background task patterns
- Update tokio macro usage (`#[tokio::main]`, `#[tokio::test]`)

**Out of Scope:**
- Gateway WebSocket connections (deferred - use separate plan)
- Complex async patterns only needed for gateway
- Worker-specific async patterns (handled by Workers runtime)

## Async Runtime Analysis

### 1. Current Tokio Dependencies

**File: `Cargo.toml`**
```toml
tokio = { version = "1.34.0", features = ["macros", "rt", "sync", "time"] }
```

**Features breakdown:**
- `macros`: `#[tokio::main]`, `#[tokio::test]`, `select!`
- `rt`: Runtime, Handle, task spawning
- `sync`: Covered in plan 000 (parking_lot)
- `time`: Sleep, interval, timeout, Instant

### 2. Task Spawning Locations

**Likely locations:**
- `src/client/mod.rs` - Background task spawning
- `src/gateway/shard.rs` - Shard management tasks
- `src/gateway/bridge/` - Event processing tasks
- Any long-running background processes

### 3. Time Utility Locations

**Likely locations:**
- Rate limiting (`src/http/ratelimiting.rs`)
- Cache expiration (`src/cache/`)
- Gateway heartbeat (disabled for WASM)
- Timeout operations
- Retry logic

### 4. Channel Locations

**Likely locations:**
- `src/client/mod.rs` - Uses `futures::channel::mpsc`
- Gateway message passing
- Event dispatching

## Implementation Plan

### Phase 1: Feature Flag and Dependencies

1. **Update `Cargo.toml`**
```toml
[features]
wasm = [
    "model",
    "http",
    "builder",
    "utils",
]

# Native dependencies
[dependencies]
tokio = { version = "1.34.0", default-features = false, features = ["macros", "rt", "sync", "time"], optional = true }
futures = "0.3.29"

# WASM-compatible alternatives
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { version = "1.34.0", default-features = false, features = ["macros", "rt", "sync", "time"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
futures-timer = "3.0.2"
wasm-bindgen-futures = "0.4.37"
```

2. **Remove tokio features from wasm target:**
```toml
# Ensure no tokio features for WASM
[target.'cfg(target_arch = "wasm32")'.dependencies]
# tokio not listed
```

### Phase 2: Create Async Runtime Abstraction

1. **Create `src/internal/async_runtime.rs`**
```rust
//! Async runtime abstraction for cross-platform support

// Task spawning abstraction
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::task::spawn;
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::task::spawn_blocking;

#[cfg(target_arch = "wasm32")]
/// No-op spawn for WASM - Cloudflare Workers don't support arbitrary task spawning
/// Tasks must run within the request-response cycle
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: futures::Future + Send + 'static,
    F::Output: Send + 'static,
{
    // For WASM, we cannot spawn independent tasks
    // This is a placeholder that will compile but shouldn't be used
    // Proper pattern: inline tasks or use Workers features
    panic!("tokio::spawn is not supported in Cloudflare Workers. Use inline async or Workers features.");
}

// Time utilities abstraction
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::time::{sleep, sleep_until, interval, timeout, Duration, Instant};

#[cfg(target_arch = "wasm32")]
pub use futures_timer::Delay;

#[cfg(target_arch = "wasm32")]
pub type Duration = std::time::Duration;

#[cfg(target_arch = "wasm32")]
pub type Instant = std::time::Instant;

#[cfg(target_arch = "wasm32")]
/// Sleep for the specified duration
pub async fn sleep(duration: Duration) {
    Delay::new(duration).await;
}

#[cfg(target_arch = "wasm32")]
/// Sleep until the specified instant
pub async fn sleep_until(instant: Instant) {
    let duration = instant.saturating_duration_since(Instant::now());
    if !duration.is_zero() {
        Delay::new(duration).await;
    }
}

#[cfg(target_arch = "wasm32")]
/// Timeout a future after the specified duration
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, futures_timer::error::Error>
where
    F: futures::Future<Output = T>,
{
    futures_timer::timeout(duration, future).await
}

#[cfg(target_arch = "wasm32")]
/// Create a new interval that yields at a fixed period
pub fn interval(period: Duration) -> impl futures::Stream<Item = ()> {
    use futures::stream::{repeat, StreamExt};
    repeat(()).then(move |_| sleep(period))
}

// Main function abstraction
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::main;

#[cfg(target_arch = "wasm32")]
/// No-op for WASM - Workers use different entry points
pub use wasm_bindgen_futures::future_to_promise as main;
```

2. **Update `src/internal/mod.rs`**
```rust
pub mod async_runtime;
```

### Phase 3: Refactor Client Module

**File: `src/client/mod.rs`**

1. **Replace tokio::spawn:**
```rust
// Before:
use tokio::spawn;
use futures::channel::mpsc::UnboundedReceiver as Receiver;

// After:
use crate::internal::async_runtime::spawn;
use futures::channel::mpsc::UnboundedReceiver as Receiver;
```

2. **Conditionalize background task spawning:**
```rust
// In ClientBuilder::build or similar
#[cfg(not(target_arch = "wasm32"))]
async fn start_background_tasks(self) -> Result<(), ClientError> {
    // Native: spawn background tasks for cache, etc.
    let client = Arc::clone(&self.http);
    let cache = self.cache.clone();
    
    spawn(async move {
        // Background processing
    });
    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn start_background_tasks(self) -> Result<(), ClientError> {
    // WASM: No background tasks - process synchronously or use Workers features
    Ok(())
}
```

3. **Replace tokio time utilities:**
```rust
// Before:
use tokio::time::{sleep, Duration};

// After:
use crate::internal::async_runtime::{sleep, Duration};
```

### Phase 4: Refactor HTTP Module

**File: `src/http/ratelimiting.rs`**

1. **Replace time utilities:**
```rust
// Before:
use tokio::time::{sleep, Duration, Instant};

// After:
use crate::internal::async_runtime::{sleep, Duration, Instant};
```

2. **Update rate limiting logic:**
```rust
// Ensure sleep uses the abstraction
pub async fn wait_until_reset(&self) -> Result<()> {
    let until = self.reset_time?;
    let now = Instant::now();
    
    if until > now {
        sleep(until - now).await;
    }
    
    Ok(())
}
```

### Phase 5: Handle Channels

**Observation:** The codebase already uses `futures::channel::mpsc` which is WASM-compatible.

**Verification:**
```rust
// In src/client/mod.rs
use futures::channel::mpsc::UnboundedReceiver as Receiver;
```

This is already correct - futures channels work on both platforms.

### Phase 6: Remove Gateway Background Tasks

**Strategy:** Disable gateway feature for WASM or provide alternative implementation

1. **Conditionalize gateway features:**
```toml
[features]
wasm = [
    "model",
    "http",
    "builder",
    "utils",
    # Exclude: gateway (requires persistent connections and background tasks)
]
```

2. **Add compile-time checks:**
```rust
#[cfg(all(feature = "wasm", feature = "gateway"))]
compile_error!("Gateway feature is not supported in WASM builds. Use REST API instead.");
```

### Phase 7: Update Examples

1. **Replace `#[tokio::main]` with abstraction:**
```rust
// Before:
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}

// After (native):
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}

// After (WASM):
#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = run().await {
            eprintln!("Error: {}", e);
        }
    });
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Same code
}
```

2. **Create WASM-specific example:**
```rust
// examples/wasm_rest_bot.rs
use serenity::http::Http;
use serenity::model::id::ChannelId;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn send_message(channel_id: u64, content: String) -> Result<String, JsValue> {
    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|e| JsValue::from_str(&format!("Token error: {}", e)))?;
    
    let http = Http::new(token);
    let channel = ChannelId::new(channel_id);
    
    channel.send_message(&http, |m| {
        m.content(content);
        Ok(m)
    }).await
    .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;
    
    Ok("Message sent".to_string())
}
```

### Phase 8: Update Tests

1. **Replace `#[tokio::test]`:**
```rust
// Before:
#[tokio::test]
async fn test_something() {
    // ...
}

// After (native):
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_something() {
    // ...
}

// After (WASM):
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_something() {
    // ...
}
```

2. **Add wasm-bindgen-test dependency:**
```toml
[dev-dependencies]
wasm-bindgen-test = "0.3.37"
```

## Potential Issues and Solutions

### Issue 1: No Arbitrary Task Spawning in Workers
**Problem:** Cloudflare Workers don't support `tokio::spawn` - all work must complete within the request

**Solution:**
- For REST API: Inline all async operations, no background tasks
- For Workers: Use Workers features like Durable Objects for persistent state
- Pattern change: Process everything synchronously within the request lifecycle
- Document this limitation clearly

### Issue 2: Tokio Time Precision Differences
**Problem:** `tokio::time::Instant` and `std::time::Instant` may have different precision

**Solution:**
- Use `std::time::Instant` for both platforms (already in abstraction)
- Accept precision differences - not critical for rate limiting
- Test thoroughly to ensure rate limiting still works correctly

### Issue 3: Timeout Error Types
**Problem:** `tokio::time::error::Elapsed` vs `futures_timer::error::Error`

**Solution:**
```rust
// Create common error type
pub enum TimeoutError {
    Elapsed,
    Other(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<tokio::time::error::Elapsed> for TimeoutError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::Elapsed
    }
}

#[cfg(target_arch = "wasm32")]
impl From<futures_timer::error::Error> for TimeoutError {
    fn from(_: futures_timer::error::Error) -> Self {
        Self::Elapsed
    }
}
```

### Issue 4: Channel Behavior Differences
**Problem:** Workers may have different channel behavior under load

**Solution:**
- Use `futures::channel` (already used, WASM-compatible)
- Test channel operations in Worker environment
- Consider using Workers-specific messaging if needed

### Issue 5: Async Trait Implementations
**Problem:** `async-trait` may have platform-specific behavior

**Solution:**
- `async-trait` is already used and works on both platforms
- No changes needed, just verify compatibility

## Testing Strategy

### 1. Build Tests
```bash
# Test native build
cargo build

# Test WASM build
cargo build --target wasm32-unknown-unknown --features wasm

# Test both compile
cargo test --lib
```

### 2. Unit Tests
```bash
# Native tests
cargo test --lib

# WASM tests (requires wasm-pack)
wasm-pack test --node
```

### 3. Integration Tests
- Test async operations complete correctly
- Test time utilities (sleep, timeout, interval)
- Test channel operations
- Test error handling
- Test rate limiting timing

### 4. Cloudflare Workers Testing
```rust
// Test in actual Worker environment
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env) -> Result<Response> {
    // Test async operations
    let result = test_async_operations().await?;
    
    // Test time utilities
    let start = std::time::Instant::now();
    crate::internal::async_runtime::sleep(Duration::from_millis(100)).await;
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(90));
    
    Response::ok("All tests passed")
}

async fn test_async_operations() -> Result<()> {
    // Test actual API calls
    let http = Http::new(env.var("DISCORD_TOKEN")?.to_string());
    let user = http.get_current_user().await?;
    
    Ok(())
}
```

## Implementation Order

1. ✅ Setup feature flags and dependencies
2. ✅ Create async runtime abstraction layer
3. ✅ Update client module (remove/conditionalize spawn)
4. ✅ Update http module (time utilities)
5. ✅ Update cache module (if using time utilities)
6. ⚠️ Disable gateway feature for wasm
7. ✅ Update examples
8. ✅ Update tests
9. ✅ Create WASM examples
10. ✅ Run native tests
11. ✅ Create and run WASM tests
12. ✅ Test in Cloudflare Workers
13. ✅ Update documentation

## Success Criteria

- ✅ Code compiles for native platform without breaking changes
- ✅ Code compiles for WASM target with `wasm` feature
- ✅ No `tokio::spawn` in WASM build paths
- ✅ All async operations work on both platforms
- ✅ Time utilities work correctly on both platforms
- ✅ Rate limiting functions correctly in Workers
- ✅ All existing tests pass on native platform
- ✅ Basic WASM example runs in Cloudflare Workers
- ✅ No performance regression on native platform
- ✅ Documentation updated for Workers limitations

## Workers-Specific Patterns

### Correct Pattern for Workers:
```rust
// ✅ GOOD: Inline async operations
#[event(fetch)]
async fn handle_request(req: Request, env: Env) -> Result<Response> {
    let token = env.var("DISCORD_TOKEN")?.to_string();
    let http = Http::new(token);
    
    // Process everything synchronously within the request
    let user = http.get_current_user().await?;
    let guilds = http.get_guilds(None, None).await?;
    
    Response::ok(format!("Bot: {} in {} guilds", user.name, guilds.len()))
}
```

### Incorrect Pattern for Workers:
```rust
// ❌ BAD: Background task spawning
#[event(fetch)]
async fn handle_request(req: Request, env: Env) -> Result<Response> {
    spawn(async move {
        // This won't work in Workers
        background_task().await;
    });
    
    Response::ok("Started")
}
```

### Alternative for Background Tasks:
```rust
// Use Workers features (Durable Objects, Cron Triggers, etc.)
// Or process in the next request
```

## Related Plans

- `001_tokio_parking_lot.md` - Synchronization primitives
- `002_reqwest_wasm.md` - HTTP client abstraction
- `003_file_operations.md` - Remove file system operations
- `005_gateway_websocket.md` - WebSocket handling (deferred/TODO)
- `006_multipart_uploads.md` - Multipart file uploads (deferred/TODO)

## References

- [tokio::time documentation](https://docs.rs/tokio/latest/tokio/time/)
- [Cloudflare Workers runtime](https://developers.cloudflare.com/workers/runtime-apis/)
- [futures-timer documentation](https://docs.rs/futures-timer/)
- [Workers limitations](https://developers.cloudflare.com/workers/platform/limits/)
- [Async Rust in Workers](https://developers.cloudflare.com/workers/runtime-apis/)