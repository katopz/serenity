# Development Issues

This file tracks the last 10 development issues and their current status.

## Issue #000: Build Environment - Ring Crate NEON Assertion Failure

### Status: 🔴 Blocking

### Description
The build fails with a NEON assertion error in the `ring` crate dependency. This error occurs during `cargo check --lib` and is preventing compilation.

**Error Details:**
```
error[E0080]: evaluation panicked: assertion failed: (CAPS_STATIC & Neon::mask()) == Neon::mask()
  --> /Users/katopz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/src/cpu/arm/darwin.rs:78:39
```

### What Happened
- Compilation fails when running `cargo check --lib`
- Error occurs in `ring-0.17.14` crate (a dependency)
- Issue appears both with and without WASM changes (verified with git stash)
- Related to ARM64 macOS detection of NEON CPU features

### Where is the Code/Test
- Not in Serenity code - this is a dependency issue
- Affected build: `cargo check --lib`
- All platforms: This is a build environment issue

### Reflection - Struggling/Solved
**Struggling:**
- The `ring` crate has CPU feature detection failing on ARM64 macOS
- This appears to be a known issue with `ring-0.17.14` on certain macOS versions
- Both `rustls_backend` and `native_tls_backend` depend on `ring`
- The "haswell" processor warnings suggest CPU detection issues

**Potential Solutions:**
1. Update `ring` to a newer version (if available)
2. Use alternative TLS backend that doesn't depend on `ring`
3. Check for macOS-specific workarounds in Cargo.toml
4. Verify Rust toolchain version compatibility

### Remaining Work
1. **Investigate dependency tree**:
   - Run `cargo tree -i ring` to identify which features pull in `ring`
   - Check if any features can be disabled to avoid `ring`
   
2. **Try alternative solutions**:
   - Check if newer `ring` version is available
   - Consider `rustls` alternatives that don't use `ring`
   - Test with different Rust toolchain versions

3. **Verify environment**:
   - Check macOS version and ARM64 support
   - Verify `cargo --version` and `rustc --version`
   - Test on different machine/environment if possible

4. **Continue with WASM work**:
   - Once build issue is resolved, proceed with testing
   - The code changes for Plan 001 are complete and ready for testing
   - All tokio::sync imports have been replaced

### How to Dev/Test
```bash
# Check dependency tree for ring
cargo tree -i ring

# Try different build configurations
cargo check --lib --no-default-features

# Check Rust toolchain
rustc --version
cargo --version

# Check for ring updates
cargo update -p ring

# Once fixed, test the changes
cargo test --lib
cargo clippy --fix --allow-dirty
```

### Notes
- This is a blocking issue for all development work on this machine
- The WASM changes themselves appear to be correct
- Build environment needs to be fixed before proceeding with further development

---

## Issue #001: Plan 001 - Tokio to Parking Lot Migration

### Status: ⏸️ Blocked (see Issue #000)

### What Happened

### Description
Replacing `tokio::sync` primitives with `parking_lot` to enable Cloudflare Workers (WASM) compatibility while maintaining native platform functionality.

### What Happened
- Added `wasm` feature flag to Cargo.toml
- Added parking_lot as required dependency for wasm32 target
- Created `src/internal/sync.rs` abstraction layer
- Updated imports in client and gateway modules
- Added oneshot channel abstraction for WASM compatibility
- Code changes complete, ready for testing once build environment is fixed

### Where is the Code/Test
- Main changes in: `src/internal/sync.rs`
- Affected modules:
  - `src/client/` (context.rs, mod.rs)
  - `src/gateway/bridge/` (shard_manager.rs, shard_queuer.rs, shard_runner.rs)
  - `src/gateway/` (shard.rs)
  - `src/framework/standard/mod.rs`
  - `src/http/` (ratelimiting.rs, typing.rs)
- Abstraction layer: `src/internal/sync.rs`
- Public API: `src/prelude.rs` updated to use sync abstraction

### Reflection - Struggling/Solved
**Solved:**
- Created platform-agnostic abstraction layer using `#[cfg(feature = "wasm")]`
- Successfully replaced all tokio::sync (Mutex, RwLock, OnceLock) imports with sync abstraction
- Added oneshot channel abstraction using platform-specific implementation:
  - WASM: `futures::channel::oneshot`
  - Native: `tokio::sync::oneshot`
- Updated `http/typing.rs` to use conditional compilation for oneshot receiver
- All code changes complete

**Struggling:**
- Build environment issue (Issue #000) prevents testing
- Cannot verify that all changes compile correctly
- Cannot run tests to confirm no regressions

### Remaining Work
1. **Resolve build environment**:
   - Fix Issue #000 (ring crate NEON assertion)
   - Enable compilation on this machine

2. **Testing** (once build is fixed):
   - Run `cargo test --lib` to verify native platform works
   - Run `cargo clippy --fix --allow-dirty` to fix warnings
   - Verify no regressions
   
3. **Code review** (once build is fixed):
   - Check for any remaining `use tokio::sync` imports
   - Ensure all documentation examples still compile
   - Verify platform-specific code paths work correctly

4. **Collector-related code**:
   - Keep `std::sync::Mutex` in collectors (it's cross-platform compatible)
   - No changes needed for collector callbacks
   
5. **Next plan preparation**:
   - Plan 002: HTTP client abstraction (reqwest vs reqwest-wasm)

### How to Dev/Test
```bash
# First, resolve build issue (see Issue #000)
cargo check --lib  # Currently failing due to ring crate

# Once build is fixed, test on native platform
cargo test --lib

# Check for warnings and issues
cargo clippy --fix --allow-dirty

# Verify specific modules compile
cargo check --lib -p serenity

# Once wasm support is complete, test with:
wasm-pack test --node --features wasm
```

### Notes
- The `parking_lot::Mutex` is synchronous, not async - this should be fine for most use cases
- For long-running critical sections, consider `futures::lock::Mutex` as alternative
- The oneshot channel in typing.rs uses platform-specific implementation
- `std::sync::Mutex` in collectors is acceptable (works on all platforms)
- All code changes are complete and ready for testing once build environment is fixed

---

*Last Updated: 2025-01-* (Blocked by Issue #000)
*Total Issues: 2*