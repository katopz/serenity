# Cloudflare Workers (WASM) Support - Master Plan

## Overview

This file serves as the master plan for adding Cloudflare Workers (WASM) support to Serenity, focusing primarily on Discord REST API functionality. See the other numbered plans for detailed implementation steps.

**Goal:** Enable Serenity to run in Cloudflare Workers for efficient, serverless Discord bot development while maintaining full compatibility with native platforms.

## Philosophy

### REST API First
- **Primary Focus:** Discord REST API (HTTP endpoints)
- **Secondary Focus:** Webhooks, Interactions (Slash Commands)
- **Deferred:** WebSocket/Gateway (persistent connections)

### Cross-Platform Compatibility
- Native platforms (Linux, macOS, Windows): No breaking changes
- WASM platforms (Cloudflare Workers): Full feature parity where possible
- Feature flags to control platform-specific code

### Minimal Impact
- No breaking changes for existing native users
- Abstraction layers to hide platform differences
- Performance maintained on native platforms

## Plans Overview

### Essential Plans (Required for WASM Support)

| Plan | Status | Priority | Description |
|------|--------|----------|-------------|
| [001_tokio_parking_lot.md](./001_tokio_parking_lot.md) | ⚠️ Not Started | HIGH | Replace tokio::sync with parking_lot for cross-platform synchronization |
| [002_reqwest_wasm.md](./002_reqwest_wasm.md) | ⚠️ Not Started | HIGH | Abstract HTTP client for reqwest (native) and reqwest-wasm (WASM) |
| [003_file_operations.md](./003_file_operations.md) | ⚠️ Not Started | HIGH | Remove/conditionalize file system operations |
| [004_async_runtime.md](./004_async_runtime.md) | ⚠️ Not Started | HIGH | Replace tokio async runtime with WASM-compatible alternatives |

### Deferred Plans (Future Work)

| Plan | Status | Priority | Description |
|------|--------|----------|-------------|
| [005_gateway_websocket.md](./005_gateway_websocket.md) | 🚧 Deferred | LOW | WebSocket/Gateway support (requires Durable Objects) |
| [006_multipart_uploads.md](./006_multipart_uploads.md) | 🚧 Deferred | LOW | Multipart file uploads (deferred - text/JSON only for now) |

## Implementation Strategy

### Phase 1: Foundation (Plans 001-004)
**Goal:** Basic WASM compatibility for REST API

1. Setup feature flags and dependencies
2. Create abstraction layers (sync, http, async runtime)
3. Remove platform-specific dependencies (fs, tokio runtime)
4. Update core modules (client, http, cache)
5. Test on both native and WASM platforms

**Expected Outcome:**
- Serenity compiles for `wasm32-unknown-unknown` target
- REST API functionality works in Cloudflare Workers
- No breaking changes for native users

### Phase 2: Testing & Examples
**Goal:** Validate implementation and provide guidance

1. Create comprehensive test suite
2. Write WASM-specific examples
3. Create Cloudflare Workers deployment guide
4. Document platform differences
5. Performance benchmarking

**Expected Outcome:**
- Full test coverage for both platforms
- Production-ready examples
- Clear documentation for developers

### Phase 3: Optimization & Features (Future)
**Goal:** Enhance WASM support based on user feedback

1. Optimize for Workers environment
2. Add Workers-specific features
3. Explore Durable Objects for advanced use cases
4. Consider gateway implementation if needed

**Expected Outcome:**
- Improved performance and reliability
- Additional platform-specific features
- Optional gateway support

## Architecture Decisions

### Feature Flags

```toml
[features]
default = ["default_no_backend", "rustls_backend"]

# WASM support
wasm = [
    "model",
    "http",
    "builder",
    "utils",
    # Exclude: gateway (deferred)
    # Exclude: cache (may be enabled later)
    # Exclude: client (may be enabled later)
]
```

### Abstraction Layer Pattern

Each abstraction follows this pattern:

```rust
// src/internal/platform_name.rs
#[cfg(not(target_arch = "wasm32"))]
pub use native_library::Type;

#[cfg(target_arch = "wasm32")]
pub use wasm_library::Type;
```

### Conditional Compilation Pattern

```rust
// Platform-specific code
#[cfg(not(target_arch = "wasm32"))]
{
    // Native-only implementation
}

#[cfg(target_arch = "wasm32")]
{
    // WASM-only implementation
}

// Compile-time checks
#[cfg(all(feature = "wasm", feature = "unsupported_feature"))]
compile_error!("This feature is not supported in WASM builds");
```

## Use Cases Enabled

### ✅ Supported (Phase 1)

- Discord Webhooks
- REST API interactions (GET, POST, PATCH, DELETE, PUT)
- Discord Interactions (Slash Commands)
- Bot command handling via HTTP
- Rate limiting
- Text/JSON request/response handling
- Configuration via environment variables

### ⚠️ Limited Support

- In-memory caching (no persistence)
- Basic logging (Workers console)
- Task spawning (inline only, no background tasks)

### ❌ Not Supported (Phase 1)

- WebSocket/Gateway connections
- Persistent state across requests
- Background task spawning
- File system operations
- Real-time event handling
- Voice functionality

## Success Criteria

### Technical Success

- [ ] Code compiles for `x86_64-unknown-linux-gnu` (native)
- [ ] Code compiles for `wasm32-unknown-unknown` (WASM)
- [ ] All existing tests pass on native platform
- [ ] WASM tests pass with `wasm-pack test`
- [ ] Text/JSON REST API operations work in Cloudflare Workers
- [ ] No performance regression on native platform
- [ ] Zero breaking changes for existing users

### Documentation Success

- [ ] Migration guide for existing users
- [ ] Getting started guide for Workers
- [ ] Platform differences documented
- [ ] API documentation updated
- [ ] Examples for both platforms
- [ ] Deployment guide for Workers

### Community Success

- [ ] User feedback collected
- [ ] Bug reports addressed
- [ ] Feature requests evaluated
- [ ] Examples shared by community
- [ ] Blog posts/tutorials written

## Development Workflow

### For Contributors

1. **Read the relevant plan** before starting work
2. **Create a branch** from `main`
3. **Implement changes** following the plan
4. **Test on both platforms**:
   ```bash
   # Native
   cargo test --lib
   
   # WASM
   wasm-pack test --node
   ```
5. **Submit PR** with reference to the plan
6. **Update the plan** with completion status

### For Maintainers

1. **Review plans** for completeness
2. **Approve implementation** order
3. **Review code changes** against plan
4. **Test thoroughly** on both platforms
5. **Merge when ready**
6. **Close the plan** and move to next

### For Users

1. **Try the WASM builds** during development
2. **Provide feedback** on issues and features
3. **Report bugs** with platform information
4. **Share examples** and best practices
5. **Ask questions** about Workers usage

## Testing Strategy

### Platform Testing

```bash
# Native platform testing
cargo test --all-features

# WASM platform testing
wasm-pack test --node --features wasm

# Cloudflare Workers testing
wrangler dev --local
```

### Test Categories

1. **Unit Tests:** Module-level functionality
2. **Integration Tests:** Cross-module interactions
3. **Platform Tests:** Platform-specific code paths
4. **E2E Tests:** Full API calls to Discord
5. **Workers Tests:** Actual Cloudflare Workers environment

### Test Coverage Goals

- Native platform: 90%+ (maintain current)
- WASM platform: 80%+ (initial target)
- Critical paths: 100% (all platforms)

## Risk Management

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking changes for native users | Low | High | Comprehensive testing, feature flags |
| Performance regression on native | Medium | High | Benchmarking, optimization |
| WASM limitations block features | High | Medium | Clear documentation, alternative approaches |
| Third-party crate incompatibility | Medium | Medium | Research upfront, fallback options |

### Operational Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Low user adoption | Medium | Medium | Focus on use cases, provide examples |
| Maintenance burden increases | High | Low | Abstraction layers, code sharing |
| Documentation becomes outdated | High | Medium | Continuous updates, examples |
| Gateway demand increases | Low | Low | Deferred plan already prepared |

## Milestones

### Milestone 1: Foundation (Week 1-2)
- [ ] All essential plans reviewed and approved
- [ ] Feature flags configured
- [ ] Abstraction layers created
- [ ] Basic compilation on both platforms

### Milestone 2: Implementation (Week 3-4)
- [ ] All plans 000-003 implemented
- [ ] Core functionality working
- [ ] Tests passing on both platforms
- [ ] Examples updated

### Milestone 3: Validation (Week 5-6)
- [ ] Comprehensive testing
- [ ] Performance benchmarking
- [ ] Documentation complete
- [ ] Examples ready

### Milestone 4: Release (Week 7-8)
- [ ] Beta release
- [ ] User feedback collection
- [ ] Bug fixes and improvements
- [ ] Stable release

## Resources

### Documentation

- [Serenity Documentation](https://docs.rs/serenity)
- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Discord API Documentation](https://discord.com/developers/docs)
- [WASM for Rust](https://rustwasm.github.io/)

### Tools

- [wasm-pack](https://rustwasm.github.io/wasm-pack/) - WASM packaging
- [wrangler](https://developers.cloudflare.com/workers/wrangler/) - Workers CLI
- [cargo-wasi](https://github.com/bytecodealliance/cargo-wasi) - WASI support

### Community

- [Serenity Discord](https://discord.gg/serenity-rs)
- [Cloudflare Developers Discord](https://discord.cloudflare.com)
- [Rust WASM Discord](https://discord.gg/rust-wasm)

## FAQ

### Q: Why not support Gateway immediately?

A: Gateway requires persistent WebSocket connections, which don't fit the Cloudflare Workers architecture. Durable Objects can provide this, but it adds significant complexity. REST API covers most bot use cases (Interactions, Webhooks), so we're focusing on that first.

### Q: Will this affect native users?

A: No. All changes use feature flags and conditional compilation. Native builds will have zero breaking changes and may even see performance improvements from parking_lot.

### Q: Can I run a full bot in Workers?

A: Yes, for most use cases. Use Slash Commands (Interactions) for user input, Webhooks for event notifications, and REST API for all other operations. Real-time event listening (Gateway) and file uploads (Multipart) are deferred.

### Q: How do I deploy to Workers?

A: Use `wrangler` to deploy. We'll provide detailed examples and a deployment guide in the documentation.

### Q: What about databases?

A: Workers have no file system, but you can use Cloudflare Workers KV, Durable Objects, or connect to external databases (PostgreSQL, Redis, etc.) via TCP sockets.

### Q: Can I upload files in Workers?

A: Not in the initial implementation. File uploads (Multipart) are deferred. For now, you can send text/JSON data. Consider using external storage services (like Cloudflare R2) and send URLs instead of files.

## Contributing

We welcome contributions! Please:

1. Read the relevant plan before starting
2. Join discussions in issues
3. Submit PRs with clear descriptions
4. Test on both platforms when possible
5. Update documentation

## License

Same as Serenity (ISC)

## Contact

- **Issues:** [GitHub Issues](https://github.com/serenity-rs/serenity/issues)
- **Discussions:** [GitHub Discussions](https://github.com/serenity-rs/serenity/discussions)
- **Discord:** [Serenity Discord](https://discord.gg/serenity-rs)

---

**Last Updated:** 2025-01-XX  
**Status:** Planning Phase  
**Next Review:** After Phase 1 completion