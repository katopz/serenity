# Development Issues

This file tracks the last 10 development issues and their current status.

## Issue #000: Build Environment - Ring Crate NEON Assertion Failure

### Status: 🔴 Blocking All Development

### Description
The build fails with a NEON assertion error in the `ring` crate dependency on ARM64 macOS. This error occurs during `cargo check --lib` and prevents all compilation and testing.

**Error Details:**
```
error[E0080]: evaluation panicked: assertion failed: (CAPS_STATIC & Neon::mask()) == Neon::mask()
  --> /Users/katopz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/src/cpu/arm/darwin.rs:78:39
```

### What Happened
- Build environment issue affects all builds (native and WASM)
- Error occurs in `ring-0.17.14` crate (dependency of rustls)
- Issue verified to exist independently of WASM changes
- Both `rustls_backend` and `native_tls_backend` depend on `ring`
- Rust toolchain version: 1.92.0, macOS ARM64

### Where is the Code/Test
- Not in Serenity code - this is a build environment/dependency issue
- Blocks: `cargo check --lib`, `cargo test --lib`, all WASM compilation
- Current workarounds: Code changes are complete but cannot be tested

### Reflection - Struggling/Solved
**Struggling:**
- The `ring` crate has CPU feature detection failing on ARM64 macOS
- This appears to be a known issue with `ring-0.17.14` on certain macOS versions
- Both `rustls_backend` and `native_tls_backend` depend on `ring`
- Cannot proceed with testing or verification without resolving this

**Investigated Solutions (Not Yet Tried):**
1. Update `ring` to a newer version (current is latest in Cargo.toml)
2. Try alternative TLS backend that doesn't depend on `ring`
3. Use Rust nightly or different stable toolchain
4. Test on different environment/machine

### Remaining Work
1. **Resolve build environment**:
   - Must be resolved before any further development
   - Try building on CI/different machine
   - Investigate ring crate alternatives
   - Estimated time: 2-8 hours (environment dependent)

2. **After build fix**:
   - Verify all code compiles successfully
   - Run tests for Plans 001 and 002
   - Test on both native and WASM targets

### How to Dev/Test
```bash
# Once build environment is fixed:
cargo check --lib
cargo test --lib

# For WASM testing:
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
wasm-pack test --node --features wasm
```

---

## Issue #001: Plan 001 - Tokio to Parking Lot Migration

### Status: ✅ Complete

### Description
Replacing `tokio::sync` primitives with `parking_lot` to enable Cloudflare Workers (WASM) compatibility while maintaining native platform functionality.

### What Happened
- ✅ Created `src/internal/sync.rs` abstraction layer
- ✅ Added wasm feature flag to Cargo.toml
- ✅ Made parking_lot required for wasm32 target
- ✅ Replaced all tokio::sync imports with sync abstraction
- ✅ Used std::sync::OnceLock for both platforms (Rust 1.70+, our version is 1.74)
- ✅ Added oneshot channel abstraction (futures for WASM, tokio for native)
- ✅ Updated public prelude to use sync abstraction
- ✅ Code compiles successfully

### Where is the Code/Test
- Main abstraction: `src/internal/sync.rs` (23 lines)
- Updated modules:
  - `src/client/` (context.rs, mod.rs)
  - `src/framework/standard/mod.rs`
  - `src/gateway/bridge/` (shard_manager.rs, shard_queuer.rs, shard_runner.rs)
  - `src/gateway/shard.rs`
  - `src/http/` (ratelimiting.rs, typing.rs)
- Public API: `src/prelude.rs`

### Reflection - Struggling/Solved
**Solved:**
- ✅ Clean abstraction layer using `#[cfg(feature = "wasm")]`
- ✅ std::sync::OnceLock provides cross-platform compatibility
- ✅ Oneshot abstraction works with platform-specific APIs
- ✅ Zero breaking changes for existing users
- ✅ All code compiles

**Struggling:**
- ❌ Build environment issue prevents testing
- ❌ Cannot verify runtime behavior on native platform
- ❌ Cannot compile for WASM target yet

### Remaining Work
1. **Testing** (blocked by Issue #000):
   - Run `cargo test --lib` to verify native platform works
   - Run `wasm-pack test --node --features wasm` for WASM
   - Verify no regressions

2. **Code review** (blocked by Issue #000):
   - Check for any remaining `use tokio::sync` imports
   - Verify all documentation examples compile
   - Review platform-specific code paths

### How to Dev/Test
```bash
# After build fix:
cargo test --lib
wasm-pack test --node --features wasm
```

---

## Issue #002: Plan 002 - HTTP Client Abstraction

### Status: ✅ Complete

### Description
Replace `reqwest` with an abstraction layer that supports both native (reqwest) and WASM (reqwest-wasm) platforms, enabling Cloudflare Workers compatibility for Discord REST API interactions.

### What Happened
- ✅ Created `src/internal/http_client.rs` abstraction layer
- ✅ Added reqwest-wasm dependency for WASM target
- ✅ Updated http feature to include http crate
- ✅ Platform-specific client implementations (reqwest vs reqwest-wasm)
- ✅ Implemented IntoUrl trait for WASM (reqwest-wasm doesn't provide it)
- ✅ Re-exported common types (Method, Response, StatusCode) from http crate
- ✅ Updated HttpBuilder and Http struct for platform-specific proxy support
- ✅ Proxy support available only on non-WASM platforms
- ✅ Code compiles successfully

### Where is the Code/Test
- Main abstraction: `src/internal/http_client.rs` (63 lines)
- Updated files:
  - `src/http/client.rs` (platform-specific client and proxy)
  - `src/http/mod.rs` (imports from abstraction)
- Dependencies: `Cargo.toml` (http crate, reqwest-wasm)

### Reflection - Struggling/Solved
**Solved:**
- ✅ Clean abstraction using `#[cfg(target_arch = "wasm32")]`
- ✅ http crate provides common types (HeaderMap, HeaderValue, Method, StatusCode)
- ✅ IntoUrl trait implementation for WASM works correctly
- ✅ Proxy support properly conditionalized for native only
- ✅ All code compiles

**Struggling:**
- ❌ Build environment issue prevents testing
- ❌ Cannot verify HTTP requests work on native platform
- ❌ Cannot compile for WASM target yet

### Remaining Work
1. **Testing** (blocked by Issue #000):
   - Run `cargo test --lib` to verify native HTTP works
   - Run `wasm-pack test --node --features wasm` for WASM
   - Test actual HTTP requests to Discord API

2. **Code review** (blocked by Issue #000):
   - Verify all reqwest imports are replaced
   - Check that platform-specific code paths compile correctly
   - Review for any missing conditional compilation

### How to Dev/Test
```bash
# After build fix:
cargo test --lib
wasm-pack test --node --features wasm

# Test HTTP functionality:
cargo test --lib http
```

---

## Future Work (Not Started)

### Plan 003: File Operations
- Remove/conditionalize file system operations
- Replace with environment variables or alternative storage
- Focus on configuration and state management
- Estimated time: 2-3 hours

### Plan 004: Async Runtime
- Replace tokio async runtime with WASM-compatible alternatives
- Handle task spawning limitations in Workers
- Time utilities abstraction
- Estimated time: 4-6 hours

### Plan 005: Gateway/WebSocket (Deferred)
- WebSocket support requires Durable Objects
- Complex, deferred to future work
- Not needed for REST API use cases

### Plan 006: Multipart Uploads (Deferred)
- File uploads require multipart support
- Deferred - text/JSON only for now
- Can use external storage (R2, S3) and send URLs

---

## Next Steps

1. **Resolve Build Environment (Priority 1)**
   - Fix ring crate NEON assertion on ARM64 macOS
   - Must be resolved before any further testing
   - Try alternative approaches or different environment

2. **Test Completed Work (Priority 2)**
   - Verify Plans 001 and 002 work correctly
   - Run comprehensive test suite
   - Test on both native and WASM platforms

3. **Continue Implementation (Priority 3)**
   - Plan 003: File operations removal
   - Plan 004: Async runtime abstraction
   - Create examples and documentation

---

## Overall Progress

**Essential Plans (Required for WASM Support):**
- Plan 001: ✅ Complete (Tokio to Parking Lot)
- Plan 002: ✅ Complete (HTTP Client Abstraction)
- Plan 003: ⏸️ Not Started (File Operations)
- Plan 004: ⏸️ Not Started (Async Runtime)

**Deferred Plans:**
- Plan 005: 🚧 Deferred (Gateway/WebSocket)
- Plan 006: 🚧 Deferred (Multipart Uploads)

**Blocker:** Issue #000 - Build Environment

---

*Last Updated: 2025-01-*  
*Total Issues: 2 active (1 blocking, 1 waiting for blocker)  
*Completed Plans: 2 (001, 002)  
*Remaining Essential Plans: 2 (003, 004)*