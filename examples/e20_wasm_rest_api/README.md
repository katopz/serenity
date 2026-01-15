# Cloudflare Workers REST API Example

This example demonstrates how to use Serenity's Discord REST API functionality in a Cloudflare Workers environment. It shows how to build a serverless Discord bot that can interact with the Discord API without needing a persistent server.

## Features

- ✅ REST API interactions (GET, POST, PATCH, DELETE, PUT)
- ✅ Bot information retrieval
- ✅ User/guild/channel data access
- ✅ Message sending via REST API
- ✅ Environment-based configuration
- ✅ Cross-platform compatibility (works on native and WASM)

## Prerequisites

- **Rust**: Latest stable version (1.74+)
- **wasm-pack**: `cargo install wasm-pack`
- **wrangler**: `npm install -g wrangler` (or install via Cloudflare)
- **Cloudflare Account**: Free tier is sufficient
- **Discord Bot**: Create one at https://discord.com/developers/applications

## Platform Differences

### Native vs Cloudflare Workers

**Native Platforms (Linux, macOS, Windows):**
- ✅ Full tokio runtime support
- ✅ Arbitrary task spawning (`tokio::spawn`)
- ✅ Background tasks and workers
- ✅ File system access
- ✅ WebSocket/Gateway connections

**Cloudflare Workers (WASM):**
- ❌ No tokio runtime (not compatible with WASM)
- ❌ No arbitrary task spawning (all work must complete within request)
- ❌ No background tasks (use Workers features like Durable Objects)
- ❌ No file system access (use environment variables, KV, R2, D1)
- ❌ No WebSocket/Gateway (use REST API, Interactions, Webhooks)
- ✅ Async operations work fine within request-response cycle
- ✅ REST API fully supported
- ✅ Interactions (Slash Commands) fully supported
- ✅ Webhooks fully supported

### Async Runtime Patterns

**❌ Wrong Pattern for Workers:**
```rust
// This WON'T work in Cloudflare Workers
tokio::spawn(async move {
    // Background task - Workers don't support this
});

#[tokio::main]
async fn main() {
    // tokio::main not compatible with WASM
}
```

**✅ Correct Pattern for Workers:**
```rust
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env) -> Result<Response> {
    // All async work happens inline within the request-response cycle
    let token = env.var("DISCORD_TOKEN")?.to_string();
    let http = Http::new(token);
    
    // Chain async operations inline
    let user = http.get_current_user().await?;
    let guilds = http.get_guilds(None, None).await?;
    
    // Return response
    Response::ok(format!("Bot: {} in {} guilds", user.name, guilds.len()))
}
```

### Time Utilities

**Native:**
```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    sleep(Duration::from_secs(1)).await;
}
```

**Workers:**
```rust
use std::time::Duration;

#[event(fetch)]
async fn fetch(req: Request, env: Env) -> Result<Response> {
    // Use JavaScript timers or Workers features
    // For rate limiting, use Workers KV or Durable Objects
    // For simple delays, use Promise-based timers
    Response::ok("Done")
}
```

## Workers-Specific Alternatives

### Rate Limiting

Instead of relying on tokio timers for rate limiting, use Workers-native solutions:

**Option 1: Workers KV (Key-Value Storage)**
```javascript
// Use KV to track rate limits
const RATE_LIMIT_KEY = "ratelimit:user:{userId}";
const MAX_REQUESTS = 100;
const WINDOW = 3600; // 1 hour in seconds

export default {
  async fetch(request, env) {
    const userId = request.headers.get('X-User-ID');
    const key = RATE_LIMIT_KEY.replace('{userId}', userId);
    
    const current = await env.RATE_LIMIT.get(key, { type: 'json' }) || { count: 0 };
    
    if (current.count >= MAX_REQUESTS) {
      return new Response('Rate limit exceeded', { status: 429 });
    }
    
    current.count++;
    await env.RATE_LIMIT.put(key, JSON.stringify(current), {
      expirationTtl: WINDOW
    });
    
    // Process request...
  }
};
```

**Option 2: Durable Objects (For distributed rate limiting)**
```javascript
// Create a rate limiter object
export class RateLimiter {
  constructor(state, env) {
    this.state = state;
    this.storage = state.storage;
  }
  
  async checkLimit(userId, maxRequests, window) {
    const key = `user:${userId}`;
    const data = await this.storage.get(key);
    const current = data ? JSON.parse(data) : { count: 0, reset: Date.now() + window };
    
    if (current.count >= maxRequests && Date.now() < current.reset) {
      return false; // Rate limited
    }
    
    if (Date.now() >= current.reset) {
      current.count = 1;
      current.reset = Date.now() + window;
    } else {
      current.count++;
    }
    
    await this.storage.put(key, JSON.stringify(current));
    return true;
  }
}
```

### Background Tasks

Since Workers don't support background task spawning, use these alternatives:

**Option 1: Cron Triggers**
```toml
# wrangler.toml
[triggers]
crons = ["*/5 * * * *"]  # Every 5 minutes
```

```javascript
export default {
  async scheduled(event, env) {
    // This runs on a schedule
    console.log("Scheduled task running");
    
    // Example: Check for expired items
    await cleanupExpiredItems(env);
  },
  
  async fetch(request, env) {
    // Regular request handler
    return new Response("OK");
  }
};
```

**Option 2: Queue Workers**
```javascript
// Use Workers Queue to offload work
export default {
  async queue(batch, env) {
    // Process messages from a queue
    for (const message of batch.messages) {
      await processMessage(message, env);
      message.ack();
    }
  },
};
```

**Option 3: Durable Objects (For persistent state)**
```javascript
// Create a stateful object that maintains connection
export class PersistentWorker {
  constructor(state, env) {
    this.state = state;
    this.env = env;
  }
  
  async fetch(request) {
    // This object maintains state across requests
    const url = new URL(request.url);
    
    switch (url.pathname) {
      case '/process':
        return this.processRequest(request);
      case '/status':
        return this.getStatus();
      default:
        return new Response('Not found', { status: 404 });
    }
  }
  
  async processRequest(request) {
    // Process and store state
    await this.state.storage.put('lastProcessed', Date.now());
    return new Response('Processed');
  }
  
  async getStatus() {
    const lastProcessed = await this.state.storage.get('lastProcessed');
    return new Response(`Last processed: ${lastProcessed}`);
  }
}
```

### File Storage

Workers have no file system, use these alternatives:

**Option 1: Workers KV (Simple Key-Value)**
```javascript
// Store configuration or cache
await env.CACHE.put('config', JSON.stringify(config));
const config = await env.CACHE.get('config', { type: 'json' });
```

**Option 2: Workers R2 (S3-compatible Object Storage)**
```javascript
// Upload file to R2
const object = await env.BUCKET.put(
  'files/example.txt',
  new TextEncoder().encode('Hello from R2!')
);

// Download file from R2
const object = await env.BUCKET.get('files/example.txt');
const text = await object.text();
```

**Option 3: External URLs**
```javascript
// Instead of uploading files, generate presigned URLs
const uploadUrl = await generateUploadUrl(env);
return new Response(JSON.stringify({ uploadUrl }));
```

### WebSocket Connections

Workers cannot maintain persistent WebSocket connections. Use these alternatives:

**Option 1: Durable Objects (For WebSocket support)**
```javascript
// Durable Objects can maintain WebSocket connections
export class WebSocketServer {
  constructor(state, env) {
    this.state = state;
    this.webSockets = [];
  }
  
  async webSocketMessage(ws, message) {
    // Handle WebSocket messages
    ws.send(`Echo: ${message}`);
  }
  
  async webSocketClose(ws, code, reason) {
    // Handle disconnection
    this.webSockets = this.webSockets.filter(w => w !== ws);
  }
}
```

**Option 2: Server-Sent Events (SSE)**
```javascript
export default {
  async fetch(request, env) {
    const { readable, writable } = new ReadableStream({
      start(controller) {
        const interval = setInterval(() => {
          controller.enqueue(`data: ${Date.now()}\n\n`);
        }, 1000);
        
        request.signal.addEventListener('abort', () => {
          clearInterval(interval);
          controller.close();
        });
      }
    });
    
    return new Response(readable, {
      headers: {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        'Connection': 'keep-alive',
      }
    });
  }
};
```

**Option 3: Discord Interactions (For Discord bots)**
```javascript
// Use Discord Interactions (Slash Commands) instead of Gateway
export default {
  async fetch(request, env) {
    if (request.method === 'POST' && request.url.includes('/interactions')) {
      const interaction = await request.json();
      
      if (interaction.type === 1) { // PING
        return new Response(JSON.stringify({ type: 1 }), {
          headers: { 'Content-Type': 'application/json' }
        });
      }
      
      if (interaction.type === 2) { // APPLICATION_COMMAND
        return new Response(JSON.stringify({
          type: 4,
          data: { content: "Hello from Workers!" }
        }), {
          headers: { 'Content-Type': 'application/json' }
        });
      }
    }
  }
};
```

## Setup

### 1. Create a Discord Application

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application"
3. Give it a name and create it
4. Go to the "Bot" tab and click "Add Bot"
5. Copy your bot token (you'll need it for configuration)

### 2. Clone and Configure

```bash
# Navigate to the example directory
cd examples/wasm_rest_api

# Copy wrangler.toml (optional, it's already configured)
# Customize the name and routes if needed
```

### 3. Set Environment Variables

**Important**: Never commit your bot token to version control!

Using wrangler CLI:
```bash
# Set your Discord bot token (required)
wrangler secret put DISCORD_TOKEN

# Optional: Set a default channel ID for testing
wrangler secret put CHANNEL_ID
```

Or via Cloudflare Dashboard:
1. Go to Workers & Pages → Settings → Variables
2. Add `DISCORD_TOKEN` as a secret (encryption enabled)
3. Optionally add `CHANNEL_ID`

## Building

### Build Locally (Testing)

```bash
# Build the WASM package
wasm-pack build --dev --target web --out-dir build

# The build artifacts will be in the `build/` directory
```

### Build for Production

```bash
# Build optimized WASM
wasm-pack build --release --target web --out-dir build
```

## Development

### Local Testing with wrangler

```bash
# Start local development server
wrangler dev

# Test endpoints at http://localhost:8787
curl http://localhost:8787/bot/info
curl http://localhost:8787/bot/user
```

### Using Miniflare (Alternative)

```bash
# Install miniflare
npm install -g miniflare

# Run locally
miniflare --wrangler-config wrangler.toml
```

## Deployment

### Deploy to Cloudflare Workers

```bash
# Deploy to your Cloudflare account
wrangler deploy

# Deploy to production environment
wrangler deploy --env production

# Deploy to development environment
wrangler deploy --env dev
```

### Remote Testing

After deployment, you'll get a URL like `https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev`

```bash
# Test your deployed worker
curl https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/bot/info
```

## API Endpoints

Once deployed, your worker provides these REST endpoints:

### Get Bot Information
```bash
GET /bot/info
```

Returns detailed application information from Discord.

**Response:**
```json
{
  "id": "123456789012345678",
  "name": "My Bot",
  "description": "A bot for testing",
  "icon": "...",
  "rpc_origins": [],
  "bot_public": true,
  "bot_require_code_grant": false
}
```

### Get Current User
```bash
GET /bot/user
```

Returns information about the bot's user account.

**Response:**
```json
{
  "id": "123456789012345678",
  "username": "MyBot",
  "discriminator": "1234",
  "avatar": "...",
  "bot": true,
  "public_flags": 64
}
```

### Get Channels
```bash
GET /channels
```

Returns a list of channels the bot has access to.

**Response:**
```json
{
  "count": 5,
  "channels": [
    {
      "id": "987654321098765432",
      "name": "general",
      "type": 0
    }
  ]
}
```

### Get Guilds
```bash
GET /guilds
```

Returns a list of guilds (servers) the bot is in.

**Response:**
```json
{
  "count": 2,
  "guilds": [
    {
      "id": "111222333444555666",
      "name": "Test Server",
      "owner": false,
      "permissions": "104320577"
    }
  ]
}
```

### Send Message
```bash
POST /message/send
Content-Type: application/json

{
  "channel_id": "987654321098765432",
  "content": "Hello from Cloudflare Workers!"
}
```

Sends a message to a specified channel.

**Response:**
```json
{
  "success": true,
  "message_id": "123456789012345678",
  "content": "Hello from Cloudflare Workers!"
}
```

## Usage Examples

### Using curl

```bash
# Get bot info
curl https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/bot/info

# Get current user
curl https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/bot/user

# Get channels
curl https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/channels

# Get guilds
curl https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/guilds

# Send a message
curl -X POST https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/message/send \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": "987654321098765432",
    "content": "Hello from Cloudflare Workers!"
  }'
```

### Using JavaScript (Browser/Node)

```javascript
// Get bot info
const response = await fetch('https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/bot/info');
const botInfo = await response.json();
console.log(botInfo);

// Send a message
const messageResponse = await fetch('https://wasm-rest-api.YOUR_SUBDOMAIN.workers.dev/message/send', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    channel_id: '987654321098765432',
    content: 'Hello from Cloudflare Workers!',
  }),
});
const result = await messageResponse.json();
console.log(result);
```

## Customization

### Adding New Endpoints

Edit `src/main.rs` and add new handler functions:

```rust
async fn handle_custom_endpoint(http: &Http) -> Result<Response> {
    // Your logic here
    Response::ok("Custom response")
}

// Update the fetch function to route to your new endpoint
async fn fetch(req: Request, env: worker::Env, ctx: worker::Context) -> Result<Response> {
    let path = req.url()?.path();
    
    match path {
        "/bot/info" => handle_bot_info(&http).await,
        "/custom" => handle_custom_endpoint(&http).await,  // Add this
        _ => Response::error("Not found", 404),
    }
}
```

### Using Additional Cloudflare Services

The `wrangler.toml` is pre-configured for optional use of:

- **KV**: Key-value storage for caching
- **D1**: SQL database for persistent data
- **R2**: Object storage for files

Uncomment and configure the appropriate sections in `wrangler.toml`.

## Limitations

### Current Limitations

- ❌ **No WebSocket/Gateway**: Workers cannot maintain persistent WebSocket connections
- ❌ **No File Uploads**: Multipart file uploads are not supported in WASM builds
- ❌ **No File System**: Workers have no file system access
- ⚠️ **Limited Task Spawning**: Background tasks must complete within the request cycle

### Workarounds

- For real-time events, use Discord's **Interactions** (Slash Commands) instead of Gateway
- For file uploads, store files in Cloudflare R2 and send URLs instead
- For persistence, use Cloudflare KV, D1, or external databases
- For scheduled tasks, use Cloudflare Cron Triggers

## Troubleshooting

### Build Errors

**Error: `error: linker 'aarch64-linux-gnu-gcc' not found`**

Make sure you have the cross-compilation tools for your target platform:
```bash
# On macOS
brew install wasm-pack

# On Linux
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Deployment Errors

**Error: `DISCORD_TOKEN environment variable not set`**

Set the secret before deploying:
```bash
wrangler secret put DISCORD_TOKEN
```

### Runtime Errors

**Error: `403 Forbidden`**

- Check that your bot token is correct
- Ensure your bot has the required permissions
- Verify the bot is invited to the guild/channel

## Best Practices

1. **Use Environment Variables**: Never hardcode credentials
2. **Error Handling**: Always handle API errors gracefully
3. **Rate Limiting**: Discord has rate limits - be aware of them
4. **Logging**: Use Cloudflare Workers logging for debugging
5. **Testing**: Test locally before deploying to production
6. **Permissions**: Only request the permissions your bot actually needs

## Resources

- [Serenity Documentation](https://docs.rs/serenity)
- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Discord API Documentation](https://discord.com/developers/docs)
- [wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

## Support

- **Issues**: [GitHub Issues](https://github.com/serenity-rs/serenity/issues)
- **Discussions**: [GitHub Discussions](https://github.com/serenity-rs/serenity/discussions)
- **Discord**: [Serenity Discord Server](https://discord.gg/serenity-rs)

## License

This example is part of Serenity and follows the same ISC license.