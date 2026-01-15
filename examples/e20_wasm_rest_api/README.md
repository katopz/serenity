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