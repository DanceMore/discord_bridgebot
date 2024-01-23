use dotenv::dotenv;
use std::env;
use std::num::NonZeroU64;
use tokio;

use serenity::all::Command;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::Interaction;
use serenity::all::Ready;

use serenity::client::{Context, EventHandler};
use serenity::framework::standard::StandardFramework;
use serenity::Client;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations() {
    let mut conn = establish_connection();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

use discord_bridgebot::establish_connection;

extern crate env_logger;
#[macro_use]
extern crate log;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("[-] inside handler");
            let content = match command.data.name.as_str() {
                "bridge" => {
                    if let Some(guild_id) = command.guild_id {
                        if let Some(guild) = guild_id.to_guild_cached(&ctx) {
                            // Now you can work with the `guild` object as expected.
                            info!("{:?} attempted to use a command...", command.user.id);
                            let guild_owner_id = guild.owner_id;

                            if command.user.id == guild_owner_id {
                                info!("{:?} appears to be a Guild Owner", guild.owner_id);
                                "I would register now (not implemented still)".to_string()
                            //		commands::bridge::run(&ctx, command.data.message, &command.data.options);
                            } else {
                                "You are not the server owner.".to_string()
                            }
                        } else {
                            "Server owner not found.".to_string()
                        }
                    } else {
                        "Failed to get guild information.".to_string()
                    }
                }
                _ => "not implemented :(".to_string(),
            };

	    // TODO: irrefutable_let_patterns, I think all my returns in this need wrapped in Some()
	    // see also: https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs
            if let content = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("[!] Cannot respond to slash command: {}", why);
                }
            }
        }
    }

    // THE MAIN BRIDGE CHAT FUNCTION
    async fn message(&self, ctx: Context, msg: Message) {
        // fast-fail to prevent spamming / looping
        if msg.author.bot {
            return;
        }

        let channel_id = msg.channel_id;
        info!(
            "[-] message spotted by EventHandler inside channel {}",
            channel_id
        );

        // Assuming msg.content contains the message
        let message = &msg.content;
        let author = &msg.author.name;

        // Take the first 16 characters
        let message_part = message.chars().take(16).collect::<String>();
        let author_part = author.chars().take(8).collect::<String>();

        // Check if the message was truncated and add "..." if necessary
        let mut padded_message = format!("{:<16}", message_part);
        if message.len() > 16 {
            padded_message.pop(); // Remove the last space
            padded_message.push_str("..."); // Add "..."
        }

        // Pad the message_part with spaces to ensure alignment
        let padded_author = format!("{:<8}", author_part);

        let results = get_channel_pairs(channel_id.into());

        if let Ok(pairs) = results {
            if pairs.is_empty() {
                // Do no work, results are empty
                debug!(
                    "[-] No pairs found for channelID {}, do no work.",
                    channel_id
                );
                return;
            }

            for pair in &pairs {
                debug!(
                    "[!] would mirror \"{}: {}\" to channel id {}",
                    padded_author, padded_message, pair.1
                );
                mirror_message(&ctx, pair.1, author, message).await;
            }
        } else {
            eprintln!("[-] Error while querying the database.");
        }
    }

    // ready up, battle bus is here ...
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("attempting to register slash command for bridgebot::bridge");
        let _ = Command::create_global_command(&ctx, commands::bridge::register()).await;
        info!("Bot is ready as {}!", ready.user.name);
    }
} // end EventHandler

//fn get_channel_pairs(channel_id: i64) -> Result<Vec<ChannelPair>, Box<dyn std::error::Error>> {
fn get_channel_pairs(channel_id: i64) -> Result<Vec<(i64, i64)>, diesel::result::Error> {
    let connection = &mut establish_connection();

    debug!("[-] db query for channel id {}", channel_id);
    use discord_bridgebot::schema::channel_pairs::dsl::*;

    // this works
    let results = channel_pairs
        .select((channel1, channel2)) // Select the columns you need
        .filter(channel1.eq(channel_id))
        .load::<(i64, i64)>(connection); // Change to the appropriate types

    //for result in &results {
    //    debug!("{:?}", result);
    //}

    results

    // this becomes scoping / type mismatch madness
    // let results = channel_pairs
    //     .select((channel1, channel2)) // Select the columns you need
    //     .filter(channel1.eq(channel_id))
    //     .load::<ChannelPair>(&mut connection)?;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    debug!("[-] hello, world, from Rust BridgeBot.");
    debug!("[-] loading config from ENV...");
    dotenv().ok();
    debug!("[+] config loaded!");

    // perform migrations
    run_migrations();

    let framework = StandardFramework::new();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

async fn mirror_message(
    ctx: &Context,
    channel_id: i64,
    custom_username: &str,
    message_content: &str,
) {
    let channel = serenity::model::id::ChannelId::from(NonZeroU64::new(channel_id as u64).unwrap());

    let message = format!("🔊 {}: {}", custom_username, message_content);

    if let Err(why) = channel.say(&ctx.http, message).await {
        eprintln!("Error sending message: {:?}", why);
    }
}
