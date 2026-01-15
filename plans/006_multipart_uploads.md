# Plan: Multipart File Uploads for WASM (DEFERRED/TODO)

## Overview
**Status:** 🚧 DEFERRED - LOW PRIORITY

This plan outlines the requirements and challenges for implementing multipart file upload functionality for Cloudflare Workers (WASM) support. **This is intentionally out of scope for the initial WASM implementation**, which focuses solely on text/JSON REST API functionality.

## Problem Statement
- Multipart file uploads require handling binary data and file streams
- File reading operations differ significantly between native and WASM platforms
- Cloudflare Workers have no file system access and limited binary handling capabilities
- Multipart form data handling varies between reqwest and reqwest-wasm
- Initial implementation focuses on text/JSON operations only

## Current Architecture Limitations

### Why This Is Deferred

1. **WASM Binary Handling:**
   - File reading from disk is not possible in Workers
   - Binary data handling in WASM is more complex
   - Need different approaches for obtaining file data

2. **Complexity:**
   - Requires abstraction between file paths and byte data
   - Different multipart implementations across platforms
   - Error handling differs significantly

3. **Text-First Approach:**
   - Most Discord bot operations are text-based
   - Slash commands, webhooks, and basic API calls don't need files
   - Simpler to implement and test text-only operations first

4. **Alternative Solutions:**
   - Use external storage (Cloudflare R2, S3, etc.)
   - Send URLs instead of files
   - Use native platform for file uploads

## Scope

**DEFERRED - NOT IN INITIAL SCOPE:**
- `src/http/multipart.rs` - Multipart form data handling
- File reading operations
- Binary data streams
- File upload API endpoints
- Multipart request building

**Out of Scope:**
- Text/JSON request handling (covered in plan 001)
- String-based API operations
- Configuration via environment variables (covered in plan 002)

## Potential Future Approaches

### Approach 1: Byte-Based Multipart (Most Viable)

**Concept:** Accept file data as bytes instead of file paths

**Pros:**
- Works in WASM (pass Vec<u8>)
- No file system access needed
- Simple abstraction layer
- Compatible with both platforms

**Cons:**
- Caller must provide bytes (not file paths)
- Requires memory for entire file
- May not suit all use cases
- Still need file reading logic on native platform

**Implementation Outline:**
```rust
// Platform-agnostic multipart builder
pub struct Multipart {
    fields: Vec<MultipartField>,
}

pub enum MultipartField {
    Text(String, String),
    Bytes(String, Vec<u8>, String), // name, data, filename
}

impl Multipart {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(MultipartField::Text(name.into(), value.into()));
        self
    }

    pub fn add_bytes(
        mut self,
        name: impl Into<String>,
        data: Vec<u8>,
        filename: impl Into<String>,
    ) -> Self {
        self.fields.push(MultipartField::Bytes(name.into(), data, filename.into()));
        self
    }

    // Platform-specific implementations
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add_file<P: AsRef<std::path::Path>>(
        mut self,
        name: impl Into<String>,
        path: P,
    ) -> Result<Self, HttpError> {
        let path = path.as_ref();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or(HttpError::Other("Invalid filename"))?
            .to_string();
        
        let data = tokio::fs::read(path).await?;
        self.fields.push(MultipartField::Bytes(name.into(), data, filename));
        
        Ok(self)
    }
}
```

### Approach 2: Stream-Based Uploads

**Concept:** Use streaming APIs for large files

**Pros:**
- Memory efficient for large files
- Better for production use cases
- Can handle files larger than memory

**Cons:**
- Complex implementation
- Streaming differs between platforms
- More error-prone
- May not be needed for typical Discord use cases

**Implementation Outline:**
```rust
use futures::Stream;

pub struct MultipartStream {
    // Stream of byte chunks
}

impl MultipartStream {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, HttpError> {
        // Create file stream
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_bytes(data: Vec<u8>) -> Self {
        // Create in-memory stream
    }
}

impl Stream for MultipartStream {
    type Item = Result<Bytes, HttpError>;
    
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, HttpError>>> {
        // Stream implementation
    }
}
```

### Approach 3: External Storage Pattern

**Concept:** Upload files to external storage first, then send URLs

**Pros:**
- No multipart needed in Workers
- Better for large files
- Reusable across platforms
- Works with Discord's file upload via URL

**Cons:**
- Requires external service dependency
- Additional API calls
- More complex workflow
- Not a direct replacement

**Implementation Pattern:**
```rust
// Upload to R2/S3 first
let file_url = upload_to_storage(file_data, filename).await?;

// Then send URL to Discord
channel.send_message(&http, |m| {
    m.content("Here's the file:");
    m.add_file(CreateAttachment::url(file_url, filename));
    Ok(m)
}).await?;
```

## Multipart Module Analysis

### Current Implementation

**File: `src/http/multipart.rs`**

Likely current implementation:
```rust
pub use reqwest::multipart::{Form, Part};

pub struct Multipart {
    form: Form,
}

impl Multipart {
    pub fn new() -> Self {
        Self { form: Form::new() }
    }

    pub fn add_file<P: AsRef<std::path::Path>>(
        mut self,
        name: impl Into<String>,
        path: P,
    ) -> Result<Self, HttpError> {
        let part = Part::file(path)?;
        self.form = self.form.part(name, part);
        Ok(self)
    }

    // ... other methods
}
```

### Required Changes

1. **Replace reqwest multipart with abstraction:**
```rust
#[cfg(not(target_arch = "wasm32"))]
use reqwest::multipart::{Form, Part};

#[cfg(target_arch = "wasm32")]
use reqwest_wasm::multipart::{Form, Part};
```

2. **Add byte-based methods:**
```rust
impl Multipart {
    pub fn add_bytes(
        mut self,
        name: impl Into<String>,
        data: Vec<u8>,
        filename: impl Into<String>,
    ) -> Result<Self, HttpError> {
        let part = Part::bytes(data).file_name(filename);
        self.form = self.form.part(name, part);
        Ok(self)
    }
}
```

3. **Conditionalize file-based methods:**
```rust
impl Multipart {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add_file<P: AsRef<std::path::Path>>(
        mut self,
        name: impl Into<String>,
        path: P,
    ) -> Result<Self, HttpError> {
        let data = tokio::fs::read(path.as_ref()).await?;
        let filename = path.as_ref().file_name()
            .and_then(|n| n.to_str())
            .ok_or(HttpError::Other("Invalid filename"))?
            .to_string();
        
        self.add_bytes(name, data, filename)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn add_file<P: AsRef<std::path::Path>>(
        self,
        _name: impl Into<String>,
        _path: P,
    ) -> Result<Self, HttpError> {
        Err(HttpError::Unsupported(
            "File operations not supported in WASM. Use add_bytes() instead."
        ))
    }
}
```

## Implementation Considerations (Future Work)

### Phase 1: Research and Proof of Concept
- Test reqwest-wasm multipart functionality
- Evaluate binary handling in WASM
- Test memory constraints
- Benchmark performance

### Phase 2: Basic Multipart Support
- Implement byte-based multipart
- Add file reading for native platform
- Test with common file types
- Document limitations

### Phase 3: Advanced Features
- Add streaming support for large files
- Optimize memory usage
- Add progress callbacks
- Support multiple files

### Phase 4: Integration
- Integrate with Discord attachment API
- Update examples
- Add comprehensive tests
- Document Workers-specific patterns

## Challenges and Risks

### Technical Challenges

1. **Memory Constraints:**
   - Workers have memory limits
   - Large files may exceed limits
   - Need size limits and validation

2. **File Size Limits:**
   - Discord has size limits (25MB for nitro, 8MB for others)
   - Workers may have stricter limits
   - Need proper error handling

3. **Binary Data Handling:**
   - Different encoding requirements
   - Content-Type handling
   - Character set issues

4. **Error Handling:**
   - Platform-specific errors
   - File reading failures
   - Network errors

### Operational Challenges

1. **Testing:**
   - Need actual files for testing
   - Test different file types
   - Test edge cases (large files, empty files, etc.)

2. **Documentation:**
   - Clear distinction between platforms
   - Examples for both approaches
   - Migration guide for users

3. **Performance:**
   - Memory usage optimization
   - Upload speed optimization
   - Handling concurrent uploads

## Recommendations

### For Initial Release
1. **❌ DO NOT** implement multipart for WASM
2. **✅ FOCUS** on text/JSON operations
3. **✅ DOCUMENT** the limitation clearly
4. **✅ PROVIDE** alternative patterns (external storage)

### For Future Consideration
1. **Evaluate** after text-only implementation is stable
2. **RESEARCH** reqwest-wasm multipart capabilities
3. **PROTOTYPE** byte-based approach first
4. **CONSIDER** streaming for large files

### Use Case Analysis

| Use Case | With Files | Without Files |
|----------|-----------|--------------|
| Send messages | ✅ With attachments | ✅ Text only |
| Embeds | ✅ With images | ✅ Text only |
| Slash commands | ✅ With file responses | ✅ Text only |
| Webhooks | ✅ With files | ✅ Text only |
| API calls | ✅ Multipart | ✅ JSON |
| Upload to channels | ✅ Files | ❌ Not possible |

**Recommendation:** Most common bot functionality works without files. Defer file uploads until specific use cases require it.

## Alternative Patterns (No Multipart Required)

### 1. External Storage + URLs
```rust
// Upload to Cloudflare R2
async fn upload_file_to_r2(file: Vec<u8>, filename: &str) -> Result<String, HttpError> {
    // Upload to R2 bucket
    // Return public URL
}

// Then send URL to Discord
channel.send_message(&http, |m| {
    m.content("Here's your file:");
    m.add_file(CreateAttachment::url(url, filename));
    Ok(m)
}).await?;
```

### 2. Text-Only Responses
```rust
// Respond with text description
channel.send_message(&http, |m| {
    m.content("File upload is not supported in this environment. Please use the web interface.");
    Ok(m)
}).await?;
```

### 3. Hybrid Approach
```rust
// Native: Use multipart
#[cfg(not(target_arch = "wasm32"))]
async fn send_with_file(channel_id, file_path) -> Result<()> {
    channel.send_message(&http, |m| {
        m.add_file(CreateAttachment::path(file_path));
        Ok(m)
    }).await?;
}

// WASM: Use URL
#[cfg(target_arch = "wasm32")]
async fn send_with_file(channel_id, file_url) -> Result<()> {
    channel.send_message(&http, |m| {
        m.add_file(CreateAttachment::url(file_url, "file.ext"));
        Ok(m)
    }).await?;
}
```

## Success Criteria (Future Implementation)

If and when multipart is implemented:

- ✅ Byte-based multipart works on both platforms
- ✅ File-based multipart works on native platform
- ✅ Memory usage stays within limits
- ✅ Error handling is comprehensive
- ✅ Test coverage for all file types
- ✅ Documentation is clear and complete
- ✅ Performance is acceptable
- ✅ Discord attachment API integration works

## Related Plans

- `001_tokio_parking_lot.md` - Synchronization primitives
- `002_reqwest_wasm.md` - HTTP client abstraction (text-only in initial phase)
- `003_file_operations.md` - Remove file system operations (deferred multipart section)
- `004_async_runtime.md` - Task spawning and async utilities
- `005_gateway_websocket.md` - WebSocket handling (deferred/TODO)

## References

- [reqwest multipart documentation](https://docs.rs/reqwest/latest/reqwest/multipart/)
- [reqwest-wasm multipart](https://docs.rs/reqwest-wasm/)
- [Discord Attachments API](https://discord.com/developers/docs/reference#uploading-files)
- [Cloudflare R2](https://developers.cloudflare.com/r2/)
- [Workers Limits](https://developers.cloudflare.com/workers/platform/limits/)

## Decision Log

**Date:** 2025-01-XX
**Decision:** Defer multipart implementation for WASM
**Reasoning:**
1. Text/JSON operations cover most Discord bot use cases
2. File uploads add significant complexity
3. Multipart handling differs between platforms
4. File reading not possible in Workers (no fs)
5. External storage pattern is viable alternative
6. Focus initial effort on stable text-only REST API

**Next Review:** After text-only implementation is stable and user feedback is collected

## Migration Guide (When Implemented)

### From Native-Only Code
```rust
// Before: Only works on native
channel.send_message(&http, |m| {
    m.add_file(CreateAttachment::path("file.txt"));
    Ok(m)
}).await?;
```

### To Cross-Platform Code
```rust
// After: Works on both platforms
#[cfg(not(target_arch = "wasm32"))]
{
    channel.send_message(&http, |m| {
        m.add_file(CreateAttachment::path("file.txt"));
        Ok(m)
    }).await?;
}

#[cfg(target_arch = "wasm32")]
{
    let data = get_file_bytes(); // Your byte source
    channel.send_message(&http, |m| {
        m.add_file(CreateAttachment::bytes(data, "file.txt"));
        Ok(m)
    }).await?;
}
```

### Best Practices
- Always provide fallback for WASM
- Validate file size before upload
- Handle errors gracefully
- Consider external storage for large files
- Document platform differences clearly