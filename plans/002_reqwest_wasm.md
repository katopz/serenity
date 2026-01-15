# Plan: Reqwest to Reqwest-WASM Abstraction for API/REST Support

## Overview
Replace `reqwest` with an abstraction layer that supports both native (reqwest) and WASM (reqwest-wasm) platforms, enabling Cloudflare Workers compatibility for Discord REST API interactions.

## Problem Statement
- `reqwest` is not compatible with WASM/Cloudflare Workers
- The entire HTTP module in serenity depends on reqwest
- Need to maintain both native and WASM builds with feature flags
- Focus on API/REST functionality (webhooks, HTTP endpoints) for initial implementation

## Scope

**Affected Modules:**
- `src/http/client.rs` - Main HTTP client implementation
- `src/http/mod.rs` - HTTP module interface
- `src/http/ratelimiting.rs` - Rate limiting logic
- `src/http/request.rs` - Request building
- `Cargo.toml` - Dependencies

**In Scope:**
- REST API interactions (GET, POST, PATCH, DELETE, PUT)
- Rate limiting
- Headers and authentication
- Error handling
- JSON request/response handling

**Out of Scope:**
- WebSocket connections (separate plan/deferred)
- Multipart file uploads (deferred - text-only for now)
- Streaming responses (deferred - text-only for now)
- Binary data handling (deferred)
- Gateway functionality

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
reqwest = { version = ">=0.11.22", default-features = false, features = ["multipart", "stream"], optional = true }

# WASM dependencies (text-only, no multipart)
reqwest-wasm = { version = "0.11", features = ["json"], optional = true }

# Conditional dependencies
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
reqwest = { version = ">=0.11.22", default-features = false, features = ["multipart", "stream"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
reqwest-wasm = { version = "0.11", features = ["json"] }
```

2. **Update existing http feature**
```toml
http = ["mime_guess", "percent-encoding", "reqwest", "reqwest-wasm"]
```

### Phase 2: Create HTTP Client Abstraction

1. **Create `src/internal/http_client.rs`**
```rust
//! HTTP client abstraction for cross-platform support

#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::{Client, ClientBuilder, Response, StatusCode, Method, header};

#[cfg(target_arch = "wasm32")]
pub use reqwest_wasm::{Client, ClientBuilder, Response, StatusCode, Method, header};

// Re-export common types
pub type HeaderMap = http::HeaderMap;
pub type HeaderValue = http::HeaderValue;
pub type Url = url::Url;

#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::IntoUrl;

#[cfg(target_arch = "wasm32")]
pub trait IntoUrl: Send + Sync {
    fn into_url(self) -> Result<Url, url::ParseError>;
}

impl IntoUrl for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(&self)
    }
}

impl IntoUrl for &str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self)
    }
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }
}
```

2. **Update `src/internal/mod.rs`**
```rust
pub mod http_client;
```

### Phase 3: Refactor HTTP Client

**File: `src/http/client.rs`**

**Changes:**

1. **Replace imports:**
```rust
// Before:
use reqwest::{Client, ClientBuilder, Response as ReqwestResponse, StatusCode};
use reqwest::header::{HeaderMap as Headers, HeaderValue};

// After:
use crate::internal::http_client::{Client, ClientBuilder, Response as ReqwestResponse, StatusCode};
use crate::internal::http_client::{HeaderMap as Headers, HeaderValue};
```

2. **Update HttpBuilder struct:**
```rust
pub struct HttpBuilder {
    #[cfg(not(target_arch = "wasm32"))]
    client: Option<Client>,
    #[cfg(target_arch = "wasm32")]
    client: Option<reqwest_wasm::Client>,
    ratelimiter: Option<Ratelimiter>,
    ratelimiter_disabled: bool,
    token: SecretString,
    #[cfg(not(target_arch = "wasm32"))]
    proxy: Option<String>,
    application_id: Option<ApplicationId>,
    default_allowed_mentions: Option<CreateAllowedMentions>,
}
```

3. **Conditionalize proxy support:**
```rust
// Add to HttpBuilder impl:
#[cfg(not(target_arch = "wasm32"))]
pub fn proxy(mut self, proxy: impl AsRef<str>) -> Self {
    self.proxy = Some(proxy.as_ref().to_string());
    self
}

// Remove proxy support for WASM build
```

4. **Update build method:**
```rust
impl HttpBuilder {
    pub fn build(self) -> Result<Http, HttpError> {
        let client = if let Some(client) = self.client {
            client
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let builder = ClientBuilder::new()
                    .use_rustls_tls()
                    .user_agent(constants::USER_AGENT);
                
                let builder = if let Some(proxy) = self.proxy {
                    builder.proxy(reqwest::Proxy::all(proxy).map_err(HttpError::from)?)
                } else {
                    builder
                };
                
                builder.build().map_err(HttpError::from)?
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                reqwest_wasm::Client::builder()
                    .build()
                    .map_err(HttpError::from)?
            }
        };
        
        Ok(Http {
            client,
            token: self.token,
            application_id: self.application_id,
            default_allowed_mentions: self.default_allowed_mentions,
            ratelimiter: self.ratelimiter,
            ratelimiter_disabled: self.ratelimiter_disabled,
        })
    }
}
```

### Phase 4: Refactor Multipart Handling

**File: `src/http/multipart.rs`**

**Changes:**

1. **Replace imports:**
```rust
// Before:
use reqwest::multipart;

// After:
#[cfg(not(target_arch = "wasm32"))]
use reqwest::multipart;

#[cfg(target_arch = "wasm32")]
use reqwest_wasm::multipart;
```

2. **Update Multipart implementation:**
```rust
// Ensure multipart creation uses the correct client type
#[cfg(not(target_arch = "wasm32"))]
pub fn create_multipart() -> multipart::Form {
    multipart::Form::new()
}

#[cfg(target_arch = "wasm32")]
pub fn create_multipart() -> multipart::Form {
    multipart::Form::new()
}
```

### Phase 5: Update Request Building

**File: `src/http/request.rs`**

**Changes:**

1. **Replace imports:**
```rust
// Before:
use reqwest::{Body, Method};

// After:
use crate::internal::http_client::{Method};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Body;

#[cfg(target_arch = "wasm32")]
type Body = reqwest_wasm::Body;
```

2. **Update request method:**
```rust
// Ensure request building is compatible with both clients
pub fn request(self) -> crate::Result<reqwest::RequestBuilder> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut builder = self.http.client.request(
            self.method,
            self.route.url(),
        );
        
        // ... rest of implementation
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        let mut builder = self.http.client.request(
            self.method,
            self.route.url(),
        );
        
        // ... rest of implementation (may need adjustments for WASM)
    }
}
```

### Phase 6: Update Rate Limiting

**File: `src/http/ratelimiting.rs`**

**Changes:**

1. **Replace any tokio-specific time operations:**
```rust
// Before:
use tokio::time::{sleep, Duration, Instant};

// After:
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{sleep, Duration, Instant};

#[cfg(target_arch = "wasm32")]
use futures_timer::Delay;
```

2. **Update sleep operations:**
```rust
#[cfg(not(target_arch = "wasm32"))]
async fn wait_until(until: Instant) {
    sleep(until - Instant::now()).await;
}

#[cfg(target_arch = "wasm32")]
async fn wait_until(duration: Duration) {
    Delay::new(duration).await;
}
```

### Phase 7: Update Error Handling

**File: `src/http/error.rs`**

**Changes:**

1. **Ensure error types are compatible:**
```rust
// Update From implementations for reqwest errors
#[cfg(not(target_arch = "wasm32"))]
impl From<reqwest::Error> for HttpError {
    fn from(err: reqwest::Error) -> Self {
        HttpError::Request(err)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<reqwest_wasm::Error> for HttpError {
    fn from(err: reqwest_wasm::Error) -> Self {
        HttpError::Request(err)
    }
}
```

## Potential Issues and Solutions

### Issue 1: API Differences Between reqwest and reqwest-wasm
**Problem:** reqwest-wasm may not have feature parity with reqwest

**Solution:**
- Document supported features for WASM
- Provide graceful degradation for unsupported features
- Add compile-time feature flags for optional functionality

### Issue 2: Streaming Responses
**Problem:** reqwest-wasm may handle streaming differently

**Solution:**
- Implement streaming only for native platform
- Provide buffered alternatives for WASM
- Document limitations clearly

### Issue 3: Client Configuration
**Problem:** reqwest-wasm may not support all client configuration options

**Solution:**
- Conditionalize client builder methods
- Skip unsupported options for WASM with warnings
- Document configuration differences

### Issue 4: Async Runtime Compatibility
**Problem:** reqwest-wasm uses different async runtime

**Solution:**
- Ensure code is runtime-agnostic
- Use `async-trait` consistently
- Avoid tokio-specific constructs in HTTP layer

### Issue 5: Multipart File Handling (Deferred)
-**Problem:** File reading differs between native and WASM
-
-**Solution (Deferred):**
-- Defer multipart support to later phase
-- Focus on text/JSON operations for now
-- Add compile-time check to prevent multipart use in WASM
-- Document limitation clearly

## Testing Strategy

### 1. Unit Tests
```bash
# Test native platform
cargo test --package serenity --lib http

# Test WASM (requires wasm-pack)
wasm-pack test --node --http
```

### 2. Integration Tests
- Test all HTTP methods (GET, POST, PATCH, DELETE, PUT)
- Test authentication with bot tokens
- Test rate limiting behavior
- Test JSON request/response handling
- Test error handling

### 3. Cloudflare Workers Testing
- Create minimal Worker example
- Test actual Discord API interactions
- Verify rate limiting works in Worker environment
- Test error scenarios

### 4. Example Verification
```rust
// examples/wasm_bot.rs
use serenity::http::Http;
use serenity::model::id::ChannelId;

async fn test_rest_api() -> Result<(), Box<dyn std::error::Error>> {
    let http = Http::new(std::env::var("DISCORD_TOKEN")?);
    
    // Test sending a message
    let channel = ChannelId::new(123456789);
    channel.send_message(&http, |m| {
        m.content("Hello from Cloudflare Workers!");
        Ok(m)
    }).await?;
    
    Ok(())
}
```

## Implementation Order

1. ✅ Setup feature flags and dependencies
-2. ✅ Create HTTP client abstraction layer
-3. ✅ Update http/client.rs
-4. ⏸️ Skip http/multipart.rs (deferred)
-5. ✅ Update http/request.rs (text-only)
-6. ✅ Update http/ratelimiting.rs
-7. ✅ Update http/error.rs
-8. ✅ Update http/mod.rs exports
-9. ✅ Update Cargo.toml features
-10. ✅ Run native tests
-11. ✅ Create WASM test (text-only)
-12. ✅ Update documentation (text-only limitation)

## Success Criteria

- ✅ Code compiles for native platform without breaking changes
- ✅ Code compiles for WASM target with `wasm` feature
-- ✅ All existing HTTP tests pass on native platform
-- ✅ Basic REST API operations work in Cloudflare Workers
-- ✅ JSON request/response handling works on both platforms
-- ✅ Rate limiting functions correctly in Worker environment
-- ✅ Error handling is consistent across platforms
-- ✅ No performance regression on native platform

## Use Cases Enabled (Text-Only)
-
-With this implementation, users can:
-- Send Discord messages (text) via webhooks
-- Interact with Discord REST API endpoints (JSON)
-- Create/modify/delete Discord resources (text-based)
-- Handle Discord bot commands via HTTP (text-based)
-- Rate limit API requests properly
-- Send and receive JSON data
-- Handle Discord Interactions (slash commands)

## Limitations (Text-Only REST API)
-
-- WebSocket/gateway functionality not included (deferred)
-- Multipart file uploads not supported in WASM (deferred)
-- Binary data handling not supported in WASM (deferred)
-- Streaming responses not supported in WASM (deferred)
-- Some advanced reqwest features may not work on WASM
-- Client configuration options limited for WASM
-- Proxy support unavailable on WASM

## Related Plans

- `001_tokio_parking_lot.md` - Synchronization primitives
- `003_file_operations.md` - Remove file system operations
- `004_async_runtime.md` - Task spawning and async utilities
- `005_gateway_websocket.md` - WebSocket handling (deferred/TODO)
- `006_multipart_uploads.md` - Multipart file uploads (deferred/TODO)

## References

- [reqwest documentation](https://docs.rs/reqwest)
- [reqwest-wasm documentation](https://docs.rs/reqwest-wasm)
- [Cloudflare Workers HTTP API](https://developers.cloudflare.com/workers/runtime-apis/fetch/)
- [Discord REST API documentation](https://discord.com/developers/docs/reference)