# Plan: Replace Tokio with Parking Lot for WASM Support

## Overview
Replace `tokio::sync` primitives with `parking_lot` to enable Cloudflare Workers (WASM) compatibility while maintaining native platform functionality.

## Problem Statement
- Tokio runtime is not compatible with WASM/Cloudflare Workers
- `tokio::sync::{Mutex, RwLock, OnceLock, Semaphore}` are used throughout the codebase
- Need to maintain both native and WASM builds with feature flags

## Scope
**Affected Modules:**
- `src/client/mod.rs` - Uses `tokio::sync::{Mutex, RwLock}`
- `src/gateway/shard.rs` - Uses `tokio::sync::Mutex`
- `src/http/ratelimiting.rs` - Likely uses tokio sync primitives
- `src/cache/mod.rs` - Uses tokio sync primitives
- Any other files with `tokio::sync` imports

**Out of Scope:**
- `tokio::spawn` / task spawning (separate plan)
- Tokio time utilities (separate plan)
- Tokio fs/io operations (separate plan)

## Implementation Plan

### Phase 1: Feature Flag Setup

1. **Update `Cargo.toml`**
```toml
[features]
wasm = [
    "model",
    "http",
    "builder",
    "utils",
]
```

2. **Add parking_lot dependency**
```toml
[dependencies]
parking_lot = { version = "0.12.1", optional = true }

# Make parking_lot required for wasm
[target.'cfg(target_arch = "wasm32")'.dependencies]
parking_lot = "0.12.1"
```

### Phase 2: Create Abstraction Layer

1. **Create `src/internal/sync.rs`**
```rust
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
```

2. **Update `src/internal/mod.rs`**
```rust
pub mod sync;
```

### Phase 3: Refactor Client Module

**File: `src/client/mod.rs`**

**Before:**
```rust
use tokio::sync::{Mutex, RwLock};
use std::sync::OnceLock;
```

**After:**
```rust
use crate::internal::sync::{Mutex, RwLock, OnceLock};
```

**Changes needed:**
- Line 26: Replace import
- Verify all usage of Mutex, RwLock work with parking_lot
- Test async method signatures (parking_lot::Mutex is synchronous)

### Phase 4: Refactor Gateway Module

**File: `src/gateway/shard.rs`**

**Before:**
```rust
use tokio::sync::Mutex;
```

**After:**
```rust
use crate::internal::sync::Mutex;
```

**Note:**
- `gateway` feature should be disabled for wasm initially
- This module will need async-aware locking patterns

### Phase 5: Refactor HTTP Module

**File: `src/http/ratelimiting.rs`**

**Actions:**
- Replace `tokio::sync` imports with `crate::internal::sync`
- Check for any tokio-specific patterns
- Verify semaphore usage if present

### Phase 6: Refactor Cache Module

**File: `src/cache/mod.rs`**

**Actions:**
- Replace `tokio::sync` imports
- Verify cache update patterns work with synchronous locks
- Consider if cache feature should be enabled for wasm

## Potential Issues and Solutions

### Issue 1: Async Locking
**Problem:** `parking_lot::Mutex` is synchronous, while `tokio::sync::Mutex` is async

**Solution:**
- Use `.lock()` instead of `.lock().await`
- For long-running critical sections, consider `futures::lock::Mutex` as alternative

```rust
// Instead of:
let mut data = mutex.lock().await;

// Use:
let mut data = mutex.lock();
```

### Issue 2: OnceLock Differences
**Problem:** `parking_lot::OnceLock` and `tokio::sync::OnceLock` have slightly different APIs

**Solution:**
- Use `OnceLock::get_or_init()` which is available in both
- Ensure all usages use common API surface

### Issue 3: RwLock Write Guards
**Problem:** Write guard lifetimes might differ

**Solution:**
- Keep guard scopes minimal
- Test thoroughly with actual async code

## Testing Strategy

### 1. Unit Tests
```bash
# Test native platform
cargo test --lib

# Test WASM (requires wasm-pack)
wasm-pack test --node
```

### 2. Integration Tests
- Create test specifically for sync primitives
- Test concurrent access patterns
- Test lock contention scenarios

### 3. Example Verification
- Run existing examples on native platform
- Create minimal wasm example to verify basic functionality

## Implementation Order

1. ✅ Setup feature flags and dependencies
2. ✅ Create abstraction layer
3. ✅ Update client module
4. ✅ Update http module
5. ⚠️ Update cache module (may need disabled for wasm)
6. ⚠️ Update gateway module (likely disable for wasm)
7. ✅ Update other modules with tokio::sync
8. ✅ Run tests
9. ✅ Update documentation

## Success Criteria

- ✅ Code compiles for native platform without breaking changes
- ✅ Code compiles for WASM target with `wasm` feature
- ✅ All existing tests pass on native platform
- ✅ Basic WASM example runs successfully
- ✅ No performance regression on native platform

## Notes

- parking_lot is generally faster than tokio::sync for native too
- This change is beneficial even outside of WASM support
- Focus on API/REST functionality first, gateway can be separate
- Keep the abstraction minimal to avoid complexity

## Related Plans

- `002_reqwest_wasm.md` - HTTP client abstraction
- `003_file_operations.md` - Remove file system operations
- `004_async_runtime.md` - Task spawning and async utilities
- `005_gateway_websocket.md` - WebSocket handling (deferred/TODO)

## References

- [parking_lot documentation](https://docs.rs/parking_lot)
- [tokio::sync documentation](https://docs.rs/tokio/latest/tokio/sync)
- [Cloudflare Workers runtime](https://developers.cloudflare.com/workers/runtime-apis/)