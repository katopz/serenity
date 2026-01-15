//! Cloudflare Workers REST API Example - Native Testing
//!
//! This main.rs provides a local testing environment for the same REST API functionality
//! that runs in Cloudflare Workers (lib.rs). Use this for development and testing before
//! deploying to production.
//!
//! # Usage
//!
//! ```bash
//! # Set your Discord token
//! export DISCORD_TOKEN="your_bot_token_here"
//!
//! # Run the example
//! cargo run --bin wasm_rest_api
//!
//! # Or with cargo directly
//! cargo run --example wasm_rest_api
//! ```
//!
//! # Commands
//!
//! - `bot-info` - Get bot application information
//! - `bot-user` - Get the bot's user information
//! - `channels <guild_id>` - Get channels in a guild
//! - `guilds` - Get all guilds the bot is in
//! - `send-message <channel_id> <message>` - Send a message to a channel
//!
//! # Parallel with Workers
//!
//! This main.rs demonstrates the same REST API operations as lib.rs, but runs on native
//! platforms with tokio runtime. The patterns are intentionally similar to help you understand
//! how the code translates between environments.
//!
//! - **Workers (lib.rs)**: Uses `#[event(fetch)]` and the `worker` crate
//! - **Native (main.rs)**: Uses `#[tokio::main]` and `tokio` runtime

use std::env;
use std::process::ExitCode;

use serenity::http::Http;
use serenity::model::id::{ChannelId, GuildId};

#[tokio::main]
async fn main() -> ExitCode {
    // Get Discord token from environment variable
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("Error: DISCORD_TOKEN environment variable not set");
            eprintln!("Please set it with: export DISCORD_TOKEN=\"your_bot_token\"");
            return ExitCode::FAILURE;
        },
    };

    // Create HTTP client
    let http = Http::new(&token);

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::SUCCESS;
    }

    let command = args[1].as_str();

    // Execute the appropriate command
    let result = match command {
        "bot-info" => handle_bot_info(&http).await,
        "bot-user" => handle_current_user(&http).await,
        "channels" => handle_channels(&http, &args).await,
        "guilds" => handle_guilds(&http).await,
        "send-message" => handle_send_message(&http, &args).await,
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            Ok(())
        },
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// Print usage information
fn print_usage() {
    println!("Cloudflare Workers REST API Example - Native Testing");
    println!();
    println!("Usage: cargo run --example wasm_rest_api <command> [args]");
    println!();
    println!("Commands:");
    println!("  bot-info                      Get bot application information");
    println!("  bot-user                      Get the bot's user information");
    println!("  channels <guild_id>           Get channels in a guild");
    println!("  guilds                        Get all guilds the bot is in");
    println!("  send-message <channel_id> <msg>  Send a message to a channel");
    println!();
    println!("Example:");
    println!("  cargo run --example wasm_rest_api bot-info");
    println!("  cargo run --example wasm_rest_api channels 123456789");
    println!("  cargo run --example wasm_rest_api send-message 123456789 \"Hello!\"");
}

/// Get bot information (application info)
async fn handle_bot_info(http: &Http) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching bot application information...");

    let app_info = http.get_current_application_info().await?;

    println!("{}", serde_json::to_string_pretty(&app_info)?);

    Ok(())
}

/// Get the bot's user information
async fn handle_current_user(http: &Http) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching current user information...");

    let user = http.get_current_user().await?;

    let response = serde_json::json!({
        "id": user.id.get(),
        "username": user.name,
        "discriminator": user.discriminator,
        "avatar": user.avatar,
        "bot": user.bot,
        "public_flags": u64::from(user.public_flags),
    });

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Get channels in a guild
async fn handle_channels(http: &Http, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        return Err("Missing guild_id. Usage: channels <guild_id>".into());
    }

    let guild_id_str = &args[2];
    let guild_id =
        guild_id_str.parse::<u64>().map_err(|_| format!("Invalid guild_id: {}", guild_id_str))?;

    println!("Fetching channels for guild {}...", guild_id);

    let channels = http.get_channels(GuildId::new(guild_id)).await?;

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

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Get all guilds the bot is in
async fn handle_guilds(http: &Http) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching guilds...");

    let guilds = http.get_guilds(None, None).await?;

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

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Send a message to a channel
async fn handle_send_message(
    http: &Http,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 4 {
        return Err("Missing arguments. Usage: send-message <channel_id> <message>".into());
    }

    let channel_id_str = &args[2];
    let channel_id = channel_id_str
        .parse::<u64>()
        .map_err(|_| format!("Invalid channel_id: {}", channel_id_str))?;

    let content = args[3].clone();

    println!("Sending message to channel {}...", channel_id);

    let message = ChannelId::new(channel_id).create_message(http).content(&content).await?;

    let response = serde_json::json!({
        "success": true,
        "message_id": message.id.get(),
        "content": content,
    });

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}
