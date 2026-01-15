//! Cloudflare Workers REST API Example
//!
//! This example demonstrates how to use Serenity's REST API in a Cloudflare Worker.
//! It shows how to:
//! - Configure your bot with environment variables
//! - Fetch bot information
//! - Send messages via REST API
//! - Handle errors appropriately
//!
//! # Environment Variables
//!
//! Set these in your `wrangler.toml` or Cloudflare Workers dashboard:
//! - `DISCORD_TOKEN`: Your Discord bot token
//! - `CHANNEL_ID`: The channel ID to send messages to (optional for this example)
//!
//! # Deployment
//!
//! ```bash
//! cd examples/wasm_rest_api
//! wrangler deploy
//! ```

use serenity::{
    http::Http,
    model::id::{ChannelId, GuildId},
};
use worker::{event, Error as WorkerError, Request, Response, Result};

/// Handle incoming fetch requests
#[event(fetch)]
async fn fetch(req: Request, env: worker::Env, _ctx: worker::Context) -> Result<Response> {
    // Get the Discord token from environment variables
    let token = match env.var("DISCORD_TOKEN") {
        Ok(var) => var.to_string(),
        Err(_) => return Response::error("DISCORD_TOKEN environment variable not set", 500),
    };

    // Create HTTP client with the token
    let http = Http::new(&token);

    // Route the request based on the URL path
    let path = req.url()?.path();

    match path {
        "/bot/info" => handle_bot_info(&http).await,
        "/bot/user" => handle_current_user(&http).await,
        "/channels" => handle_channels(&http, req).await,
        "/guilds" => handle_guilds(&http).await,
        "/message/send" => handle_send_message(&http, req).await,
        _ => Response::error("Not found", 404),
    }
}

/// Get bot information (application info)
async fn handle_bot_info(http: &Http) -> Result<Response> {
    match http.get_current_application_info().await {
        Ok(app_info) => {
            let json = serde_json::to_string_pretty(&app_info)
                .map_err(|e| WorkerError::from(e.to_string()))?;
            Response::from_json(&json)
        },
        Err(e) => Response::error(format!("Failed to get bot info: {}", e), 500),
    }
}

/// Get the bot's user information
async fn handle_current_user(http: &Http) -> Result<Response> {
    match http.get_current_user().await {
        Ok(user) => {
            let response = serde_json::json!({
                "id": user.id.get(),
                "username": user.name,
                "discriminator": user.discriminator,
                "avatar": user.avatar,
                "bot": user.bot,
                "public_flags": u64::from(user.public_flags),
            });
            Response::from_json(&response)
        },
        Err(e) => Response::error(format!("Failed to get user info: {}", e), 500),
    }
}

/// Get channels the bot has access to
async fn handle_channels(http: &Http, req: Request) -> Result<Response> {
    // Parse the request body to get guild_id
    let body = match req.text().await {
        Ok(b) => b,
        Err(e) => return Response::error(format!("Failed to read request body: {}", e), 400),
    };

    let data: serde_json::Value = match serde_json::from_str(&body) {
        Ok(d) => d,
        Err(e) => return Response::error(format!("Invalid JSON: {}", e), 400),
    };

    // Get guild_id from request body
    let guild_id_str = data
        .get("guild_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WorkerError::from("Missing guild_id"))?;

    let guild_id = match guild_id_str.parse::<u64>() {
        Ok(id) => GuildId::new(id),
        Err(e) => return Response::error(format!("Invalid guild_id: {}", e), 400),
    };

    // Get channels for the guild
    match http.get_channels(guild_id).await {
        Ok(channels) => {
            let response = serde_json::json!({
                "count": channels.len(),
                "channels": channels.into_iter()
                    .map(|c| serde_json::json!({
                        "id": c.id.get(),
                        "name": c.name,
                        "type": c.kind,
                    }))
                    .collect::<Vec<_>>()
            });
            Response::from_json(&response)
        },
        Err(e) => Response::error(format!("Failed to get channels: {}", e), 500),
    }
}

/// Get guilds (servers) the bot is in
async fn handle_guilds(http: &Http) -> Result<Response> {
    match http.get_guilds(None, None).await {
        Ok(guilds) => {
            let response = serde_json::json!({
                "count": guilds.len(),
                "guilds": guilds.into_iter()
                    .map(|g| serde_json::json!({
                        "id": g.id.get(),
                        "name": g.name,
                        "owner": g.owner,
                        "permissions": g.permissions,
                    }))
                    .collect::<Vec<_>>()
            });
            Response::from_json(&response)
        },
        Err(e) => Response::error(format!("Failed to get guilds: {}", e), 500),
    }
}

/// Send a message to a channel
async fn handle_send_message(http: &Http, req: Request) -> Result<Response> {
    // Parse the request body to get message details
    let body = match req.text().await {
        Ok(b) => b,
        Err(e) => return Response::error(format!("Failed to read request body: {}", e), 400),
    };

    let data: serde_json::Value = match serde_json::from_str(&body) {
        Ok(d) => d,
        Err(e) => return Response::error(format!("Invalid JSON: {}", e), 400),
    };

    // Get channel ID from request body
    let channel_id_str = data
        .get("channel_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WorkerError::from("Missing channel_id"))?;

    let channel_id = match channel_id_str.parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(e) => return Response::error(format!("Invalid channel_id: {}", e), 400),
    };

    // Get message content from request body
    let content =
        data.get("content").and_then(|v| v.as_str()).unwrap_or("Hello from Cloudflare Workers!");

    // Send the message
    match channel_id.create_message(http).content(content).await {
        Ok(message) => {
            let response = serde_json::json!({
                "success": true,
                "message_id": message.id.get(),
                "content": content,
            });
            Response::from_json(&response)
        },
        Err(e) => Response::error(format!("Failed to send message: {}", e), 500),
    }
}
