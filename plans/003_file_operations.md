# Plan: Remove/Conditionalize File Operations for WASM Support

## Overview
Remove or conditionalize all file system operations to enable Cloudflare Workers (WASM) compatibility, as Workers don't have access to a traditional file system.

## Problem Statement
- Cloudflare Workers have no file system access
- Tokio fs features are enabled in serenity
- File operations may be used for configuration, caching, or logging
- Need to maintain both native and WASM builds with feature flags
- Focus on API/REST functionality - file operations are likely minimal for REST API use

## Scope

**Affected Areas:**
- `Cargo.toml` - Tokio fs features
- Any modules using `tokio::fs` or `std::fs`
- Configuration loading (if file-based)
- Cache persistence (if file-based)
- Logging to files (if present)
- Any file path utilities

**In Scope:**
- Identify all file operations
- Remove unnecessary file operations for REST API
- Conditionalize essential file operations
- Replace with in-memory or Workers-compatible alternatives
- Update dependencies
- Focus on text/JSON operations only

**Out of Scope:**
- Gateway-specific file operations (deferred)
- Examples that require file operations
- File-based configuration systems (use environment variables instead)
- Multipart file uploads (deferred - text-only for now)
- Binary file handling (deferred)

## File Operation Analysis

### 1. Tokio FS Features
**File: `Cargo.toml`**

Current tokio features include:
```toml
tokio = { version = "1.34.0", features = ["fs", "macros", "rt", "sync", "time", "io-util"] }
```

The `fs` and `io-util` features enable file operations.

### 2. Likely File Operation Locations

**Potential locations based on typical Discord bot patterns:**
- Configuration file loading (`config.json`, `.env`)
- Token reading from files
- Cache persistence to disk
- Log file writing
- Temporary file handling
- Asset file reading (for bot resources)

### 3. REST API Specific Analysis (Text-Only)

For text-based API/REST functionality, file operations are typically:
- **Token handling**: Usually via environment variables (compatible with Workers)
- **Configuration**: Can be replaced with environment variables
- **File uploads**: DEFERRED - Not supported in initial WASM implementation
- **Response handling**: In-memory JSON/text (compatible)
- **Caching**: In-memory (compatible)

## Implementation Plan

### Phase 1: Audit File Operations

1. **Search for file operation patterns:**
```bash
# Search for std::fs usage
grep -r "std::fs" src/

# Search for tokio::fs usage
grep -r "tokio::fs" src/

# Search for File, PathBuf, etc.
grep -r "use std::path::Path" src/
grep -r "use std::path::PathBuf" src/

# Search for file opening
grep -r "File::open" src/
grep -r "File::create" src/
```

2. **Document findings:**
- List all files with file operations
- Categorize by purpose (config, cache, logs, etc.)
- Determine if each is essential for REST API
- Identify removal vs. conditionalization strategy

### Phase 2: Update Dependencies

**File: `Cargo.toml`**

1. **Remove fs features for wasm:**
```toml
[dependencies]
tokio = { version = "1.34.0", default-features = false, features = ["macros", "rt", "sync", "time"] }

# Native-only features
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { version = "1.34.0", default-features = false, features = ["macros", "rt", "sync", "time", "fs", "io-util"] }

# WASM-compatible features (no fs)
[target.'cfg(target_arch = "wasm32")'.dependencies]
tokio = { version = "1.34.0", default-features = false, features = ["macros", "rt", "sync", "time"] }
```

2. **Add wasm feature flag:**
```toml
[features]
wasm = [
    "model",
    "http",
    "builder",
    "utils",
]
```

### Phase 3: Handle Configuration

**Strategy:** Use environment variables instead of config files

1. **Create `src/internal/config.rs`:**
```rust
//! Configuration abstraction for cross-platform support

use std::env;
use secrecy::{SecretString, ExposeSecret};

/// Get Discord token from environment variable
pub fn get_token() -> Result<SecretString, ConfigError> {
    env::var("DISCORD_TOKEN")
        .map(SecretString::new)
        .map_err(ConfigError::TokenNotFound)
}

/// Get application ID from environment variable
pub fn get_application_id() -> Result<u64, ConfigError> {
    env::var("DISCORD_APPLICATION_ID")
        .map_err(ConfigError::ApplicationIdNotFound)?
        .parse()
        .map_err(ConfigError::ParseError)
}

#[derive(Debug)]
pub enum ConfigError {
    TokenNotFound,
    ApplicationIdNotFound,
    ParseError(String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenNotFound => write!(f, "DISCORD_TOKEN environment variable not found"),
            Self::ApplicationIdNotFound => write!(f, "DISCORD_APPLICATION_ID environment variable not found"),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}
```

2. **Update documentation to use environment variables:**
```markdown
# Environment Variables

Set these in your `.env` file or Cloudflare Workers environment:

- `DISCORD_TOKEN`: Your bot token
- `DISCORD_APPLICATION_ID`: Your application ID (optional)
```

### Phase 4: Remove File-Based Caching

**Strategy:** Use in-memory caching for both platforms

1. **Verify cache implementation:**
- Check if cache writes to disk
- Ensure cache is in-memory only for WASM

2. **Update cache settings if needed:**
```rust
// In src/cache/mod.rs
#[cfg(feature = "wasm")]
const DEFAULT_CACHE_SETTINGS: CacheSettings = CacheSettings {
    // Disable file persistence for WASM
    max_messages: None,
    // ... other settings
};
```

### Phase 5: Handle File Uploads

**Strategy:** Defer multipart support - focus on text/JSON only

1. **Add compile-time check for multipart in WASM:**
```rust
#[cfg(all(target_arch = "wasm32", feature = "multipart"))]
compile_error!("Multipart file uploads are not supported in WASM builds. Use text/JSON only.");
```

2. **Document multipart limitation:**
- Multipart file uploads are not supported in WASM builds
- Use text/JSON payloads only
- For file uploads, use native platform or external service

### Phase 6: Remove Logging to Files

**Strategy:** Use Workers logging or console logging

1. **Check for file logging:**
- Search for logging crate usage
- Check for file appenders

2. **Update logging configuration:**
```rust
// In examples or main
#[cfg(not(target_arch = "wasm32"))]
fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
}

#[cfg(target_arch = "wasm32")]
fn init_logging() {
    // Workers have built-in logging
    // or use console_log
    console_log::init().ok();
}
```

### Phase 7: Conditionalize Any Remaining File Operations

**Strategy:** Use feature flags to disable file operations for WASM

1. **Pattern for conditional file operations:**
```rust
#[cfg(not(target_arch = "wasm32"))]
async fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(path).await
}

#[cfg(target_arch = "wasm32")]
async fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "File operations not supported in WASM",
    ))
}
```

2. **Provide compile-time errors for unsupported operations:**
```rust
#[cfg(all(feature = "wasm", feature = "file_operations"))]
compile_error!("File operations are not supported in WASM builds");
```

### Phase 8: Update Examples and Documentation

1. **Remove file-based examples:**
- Or add `#[cfg(not(target_arch = "wasm32"))]` to examples
- Create WASM-specific examples

2. **Create WASM example:**
```rust
// examples/wasm_rest_api.rs
use serenity::http::Http;
use serenity::model::id::ChannelId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use environment variables instead of config files
    let token = std::env::var("DISCORD_TOKEN")?;
    let http = Http::new(token);
    
    // Send a message via REST API
    let channel = ChannelId::new(123456789);
    channel.send_message(&http, |m| {
        m.content("Hello from WASM!");
        Ok(m)
    }).await?;
    
    Ok(())
}
```

## Potential Issues and Solutions

### Issue 1: Examples Using File Paths
**Problem:** Existing examples may use file paths for tokens or configuration

**Solution:**
- Update examples to use environment variables
- Add conditional compilation to skip examples on WASM
- Create separate WASM examples

### Issue 2: Asset Files
**Problem:** Bots may read images, sounds, or other assets

**Solution:**
- For WASM: Use Cloudflare R2 or Workers KV for assets
- For native: Keep file reading
- Provide asset abstraction layer

### Issue 3: Temporary Files
**Problem:** Some operations may create temporary files

**Solution:**
- Use in-memory alternatives
- Remove temporary file usage if possible
- Document limitation for WASM

### Issue 4: File System Testing
**Problem:** Tests may rely on file operations

**Solution:**
- Use `#[cfg(not(target_arch = "wasm32"))]` for file-based tests
- Create alternative tests for WASM
- Use mock data instead of files

## Testing Strategy

### 1. Build Tests
```bash
# Test native build with fs
cargo build

# Test WASM build without fs
cargo build --target wasm32-unknown-unknown --features wasm

# Test both work
cargo test --lib
```

### 2. Integration Tests
- Test REST API operations without files
- Test configuration via environment variables
- Test text/JSON request/response handling
- Test error handling when files are not available

### 3. Cloudflare Workers Testing
```javascript
// wrangler.toml
[vars]
DISCORD_TOKEN = "your_token_here"
DISCORD_APPLICATION_ID = "your_app_id"
```

```rust
// Test in actual Worker environment
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env) -> Result<Response> {
    let token = env.var("DISCORD_TOKEN")?.to_string();
    let http = Http::new(token);
    
    // Test API call
    let result = http.get_current_user().await?;
    
    Response::ok(format!("Bot: {}", result.name))
}
```

## Implementation Order

1. ✅ Audit all file operations in codebase
2. ✅ Update Cargo.toml dependencies
3. ✅ Create configuration abstraction
4. ✅ Remove/conditionalize file-based config loading
5. ✅ Verify cache doesn't use file system
6. ✅ Verify multipart uses bytes, not file paths
7. ✅ Remove or conditionalize file logging
8. ✅ Conditionalize any remaining file operations
9. ✅ Update examples
10. ✅ Update documentation
11. ✅ Create WASM examples
12. ✅ Test both builds
13. ✅ Test in Cloudflare Workers

## Success Criteria

- ✅ No `fs` or `io-util` features in tokio for WASM builds
- ✅ Code compiles for native platform without breaking changes
- ✅ Code compiles for WASM target with `wasm` feature
- ✅ No file operations in WASM build paths
- Configuration uses environment variables
- REST API functionality works without file system
- Text/JSON operations work on both platforms
- All existing tests pass on native platform
- Basic WASM example runs in Cloudflare Workers
- Documentation updated for Workers environment (text-only)

## Workers-Specific Alternatives

For operations that would normally use files:

| Operation | Native | Workers |
|-----------|--------|---------|
| Configuration | File (config.json) | Environment variables / Secrets |
| Token storage | File (.env) | Environment variable |
| Cache persistence | Disk | In-memory / KV / Durable Objects |
| Assets | File system | R2 / KV / CDN |
| Logs | File | Workers logging / Console |
| Temp files | /tmp | In-memory / None |
| File uploads | Multipart bytes | ❌ Deferred (use external service) |
| Text/JSON data | File or String | String / Environment variables |

## Related Plans

- `001_tokio_parking_lot.md` - Synchronization primitives
- `002_reqwest_wasm.md` - HTTP client abstraction
- `004_async_runtime.md` - Task spawning and async utilities
- `005_gateway_websocket.md` - WebSocket handling (deferred/TODO)
- `006_multipart_uploads.md` - Multipart file uploads (deferred/TODO)

## References

- [Cloudflare Workers Limits](https://developers.cloudflare.com/workers/platform/limits/)
- [Workers KV Storage](https://developers.cloudflare.com/kv/)
- [Workers R2 Storage](https://developers.cloudflare.com/r2/)
- [Workers Environment Variables](https://developers.cloudflare.com/workers/configuration/environment-variables/)
- [tokio::fs documentation](https://docs.rs/tokio/latest/tokio/fs/)