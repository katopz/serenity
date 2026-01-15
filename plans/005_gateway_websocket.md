# Plan: WebSocket/Gateway Support for WASM (DEFERRED/TODO)

## Overview
**Status:** 🚧 DEFERRED - LOW PRIORITY

This plan outlines the requirements and challenges for implementing Discord Gateway (WebSocket) functionality for Cloudflare Workers (WASM) support. **This is intentionally out of scope for the initial WASM implementation**, which focuses solely on API/REST functionality.

## Problem Statement
- Discord Gateway requires persistent WebSocket connections for real-time events
- Cloudflare Workers have architectural limitations that prevent traditional persistent WebSocket server connections
- The entire gateway module depends on tokio runtime and persistent connections
- Complex architecture changes would be required to support gateway in Workers

## Current Architecture Limitations

### Why This Is Deferred

1. **Workers Architecture:**
   - Workers are designed for request-response cycles
   - They don't support long-lived server-side WebSocket connections
   - Traditional Discord bot patterns don't apply to Workers

2. **Complexity:**
   - Requires significant architectural changes
   - Would need Workers-specific features (Durable Objects)
   - Implementation effort is high for limited use cases

3. **REST API First:**
   - Most bot functionality can be achieved via REST API
   - Interactions (slash commands) work via HTTP endpoints
   - Webhooks can receive events without gateway

4. **Alternative Solutions:**
   - Use traditional bot server for gateway
   - Use Workers as a proxy/frontend
   - Durable Objects provide alternative (but complex)

## Scope

**DEFERRED - NOT IN INITIAL SCOPE:**
- `src/gateway/mod.rs` - All gateway functionality
- `src/gateway/shard.rs` - WebSocket connection management
- `src/gateway/ws.rs` - WebSocket client
- `src/gateway/bridge/` - Event processing
- `src/client/mod.rs` - Gateway integration
- All WebSocket-related code

**Out of Scope:**
- REST API functionality (covered in other plans)
- Discord Interactions via HTTP
- Webhook-based event handling
- All features that work without gateway

## Potential Future Approaches

### Approach 1: Durable Objects (Most Viable)

**Concept:** Use Cloudflare Durable Objects for persistent WebSocket connections

**Pros:**
- Durable Objects support WebSocket connections
- Provide stateful, persistent storage
- Can maintain gateway connection
- Compatible with Workers ecosystem

**Cons:**
- Requires architectural changes
- Durable Objects are a separate service
- More complex deployment
- Additional cost considerations
- Not yet fully stable in all regions

**Implementation Outline:**
```rust
// Gateway as a Durable Object
use worker::*;

#[durable_object]
pub struct DiscordGateway {
    client: Option<WebSocket>,
    token: String,
    intents: GatewayIntents,
    // ... state management
}

impl DurableObject for DiscordGateway {
    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // Handle WebSocket upgrade
        if web_sys::WebSocket::new("").is_ok() {
            // ... WebSocket handling
        }
        
        Ok(Response::ok("Gateway running"))
    }
}
```

### Approach 2: Worker-to-Worker WebSocket

**Concept:** Spawn a separate Worker for WebSocket connections

**Pros:**
- Can maintain persistent connection
- Separates concerns
- Uses standard WebSocket APIs

**Cons:**
- Requires coordination between Workers
- Still has timeout limitations
- Complex state management
- Higher operational complexity

**Implementation Outline:**
```javascript
// Separate WebSocket worker
export default {
  async fetch(request, env, ctx) {
    const upgradeHeader = request.headers.get("Upgrade");
    if (upgradeHeader === "websocket") {
      return handleWebSocket(request, env);
    }
    return new Response("Expected WebSocket", { status: 426 });
  }
};
```

### Approach 3: Hybrid Architecture

**Concept:** Traditional bot server for gateway + Workers for REST API

**Pros:**
- Leverages existing gateway implementation
- Workers handle REST API efficiently
- Minimal changes to existing code
- Clear separation of concerns

**Cons:**
- Requires running two services
- Not "pure" Workers solution
- Additional deployment complexity
- Higher operational overhead

**Architecture:**
```
Discord Gateway ←→ Traditional Bot Server (Rust on VPS)
                            ↓
                      Events/Commands
                            ↓
Cloudflare Workers ←→ REST API (HTTP)
```

### Approach 4: Interactions/Webhooks Only

**Concept:** Don't use gateway at all

**Pros:**
- Fully compatible with Workers
- Simplest architecture
- Lower operational complexity
- Sufficient for many use cases

**Cons:**
- Limited event types
- No real-time updates
- Can't listen to all events
- Different development pattern

**What You Can Do:**
- Slash commands via Interactions
- Webhook-based event notifications
- REST API for all operations
- No persistent connection needed

## Gateway Module Analysis

### Current Dependencies

**Files requiring changes:**
- `src/gateway/shard.rs` - WebSocket client, tokio-tungstenite
- `src/gateway/ws.rs` - WebSocket implementation
- `src/gateway/mod.rs` - Gateway coordination
- `src/gateway/bridge/` - Event handling
- `src/client/mod.rs` - Gateway integration

**Dependencies to replace:**
- `tokio-tungstenite` → WASM-compatible WebSocket library
- `tokio::sync` → `parking_lot` (covered in plan 000)
- `tokio::spawn` → Durable Objects or alternative
- Persistent state management → Workers KV/Durable Objects

### Key Components to Address

1. **WebSocket Connection:**
   ```rust
   // Current: tokio-tungstenite
   use tokio_tungstenite::tungstenite::client::IntoClientRequest;
   
   // Future: web-sys or wasm-tungstenite
   #[cfg(target_arch = "wasm32")]
   use web_sys::WebSocket;
   ```

2. **Heartbeat Management:**
   ```rust
   // Current: tokio timer
   use tokio::time::{interval, sleep};
   
   // Future: workers timer or manual polling
   ```

3. **Event Loop:**
   ```rust
   // Current: tokio select!
   tokio::select! {
       Some(message) = ws.next() => { /* ... */ },
       _ = heartbeat.tick() => { /* ... */ },
   }
   
   // Future: event-based or polling
   ```

4. **Shard Management:**
   - Sharding in Workers is complex
- May need alternative approach
- Durable Objects could help

## Implementation Considerations (Future Work)

### Phase 1: Research and Proof of Concept
- Test Durable Objects with WebSocket
- Evaluate wasm-tungstenite compatibility
- Test Workers WebSocket limits
- Benchmark performance

### Phase 2: Minimal Gateway
- Implement single shard connection
- Handle basic events
- Test with simple bot
- Document limitations

### Phase 3: Full Gateway
- Implement sharding
- Handle all events
- Add resume/reconnect logic
- Optimize for Workers

### Phase 4: Integration
- Integrate with client module
- Update examples
- Add documentation
- Test thoroughly

## Challenges and Risks

### Technical Challenges

1. **Connection Stability:**
   - Workers have execution time limits
   - WebSocket may timeout
   - Need robust reconnection logic

2. **State Management:**
   - Gateway state must persist
   - Cannot rely on memory
   - Need Durable Objects or KV

3. **Sharding Complexity:**
   - Multiple WebSocket connections
   - Coordination between Workers
   - Increased complexity

4. **Heartbeat Issues:**
   - Timing precision in Workers
   - May need alternative approach
   - Risk of disconnection

### Operational Challenges

1. **Cost:**
   - Durable Objects have pricing
   - Additional infrastructure
   - Higher than pure REST

2. **Monitoring:**
   - Need visibility into gateway
   - Workers monitoring limitations
   - Complex debugging

3. **Deployment:**
   - Multiple services to deploy
   - Coordination required
   - CI/CD complexity

4. **Maintenance:**
   - Two different runtimes
   - Version compatibility
   - Ongoing effort

## Recommendations

### For Initial Release
1. **❌ DO NOT** implement gateway for WASM
2. **✅ FOCUS** on REST API functionality
3. **✅ USE** Discord Interactions for commands
4. **✅ LEVERAGE** Webhooks for events

### For Future Consideration
1. **Evaluate** after REST API is stable
2. **Research** Durable Objects thoroughly
3. **PROTOTYPE** before full implementation
4. **CONSIDER** hybrid architecture alternative

### Use Case Analysis

| Use Case | With Gateway | REST API Only |
|----------|-------------|---------------|
| Slash commands | ✅ | ✅ |
| Message commands | ✅ | ✅ |
| Webhook messages | ✅ | ✅ |
| Real-time chat | ✅ | ❌ |
| Member tracking | ✅ | ❌ |
| Voice states | ✅ | ❌ |
| Bot status | ✅ | ⚠️ Limited |
| Event logging | ✅ | ⚠️ Limited |

**Recommendation:** Most common bot functionality works without gateway. Defer gateway implementation until specific use cases require it.

## Alternative Patterns (No Gateway Required)

### 1. Discord Interactions
```rust
// Handle slash commands via HTTP
#[event(fetch)]
async fn handle_interaction(req: Request, env: Env) -> Result<Response> {
    let interaction = req.json::<Interaction>().await?;
    let http = Http::new(env.var("DISCORD_TOKEN")?.to_string());
    
    match interaction.kind {
        InteractionType::ApplicationCommand => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Hello!")
            );
            interaction.create_response(&http, response).await?;
        }
        _ => {}
    }
    
    Response::ok("OK")
}
```

### 2. Webhooks
```rust
// Receive events via webhooks
#[event(fetch)]
async fn handle_webhook(req: Request, env: Env) -> Result<Response> {
    let event = req.json::<WebhookEvent>().await?;
    
    // Process event
    match event {
        WebhookEvent::MessageCreate(msg) => {
            // Handle new message
        }
        _ => {}
    }
    
    Response::ok("OK")
}
```

### 3. Scheduled Tasks
```rust
// Use Workers Cron Triggers
#[event(scheduled)]
async fn handle_scheduled(event: ScheduledEvent, env: Env) -> Result<()> {
    let http = Http::new(env.var("DISCORD_TOKEN")?.to_string());
    
    // Perform periodic tasks
    // e.g., check for reminders, update status, etc.
    
    Ok(())
}
```

## Success Criteria (Future Implementation)

If and when gateway is implemented:

- ✅ Stable WebSocket connection via Durable Objects
- ✅ Support for at least basic Discord events
- ✅ Automatic reconnection on disconnect
- ✅ Heartbeat management
- ✅ Compatible with existing event handlers
- ✅ Test coverage for all gateway features
- ✅ Documentation for Workers-specific patterns
- ✅ Performance comparable to native platform
- ✅ Cost-effective operation

## Related Plans

- `001_tokio_parking_lot.md` - Synchronization primitives (completed)
- `002_reqwest_wasm.md` - HTTP client abstraction (completed)
- `003_file_operations.md` - Remove file system operations (completed)
- `004_async_runtime.md` - Task spawning and async utilities (completed)

## References

- [Cloudflare Durable Objects](https://developers.cloudflare.com/durable-objects/)
- [Workers WebSocket](https://developers.cloudflare.com/workers/runtime-apis/websockets/)
- [Discord Gateway Documentation](https://discord.com/developers/docs/topics/gateway)
- [Durable Objects WebSocket Guide](https://developers.cloudflare.com/durable-objects/api/websockets/)
- [Workers Limits](https://developers.cloudflare.com/workers/platform/limits/)

## Decision Log

**Date:** 2025-01-XX
**Decision:** Defer gateway implementation for WASM
**Reasoning:**
1. REST API provides sufficient functionality for most use cases
2. Gateway implementation in Workers is complex and requires Durable Objects
3. Focus initial effort on stable REST API support
4. Revisit when specific use cases require real-time features
5. Alternative patterns (Interactions, Webhooks) work well in Workers

**Next Review:** After REST API stabilization and user feedback