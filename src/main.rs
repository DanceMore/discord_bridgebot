use dotenv::dotenv;
use std::env;
use std::num::NonZeroU64;
use tokio;

use serenity::all::Ready;
use serenity::client::EventHandler;
use serenity::Client;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::commands::bridge::bridge;

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
    // THE MAIN BRIDGE CHAT FUNCTION
    async fn message(&self, ctx: poise::serenity_prelude::Context, msg: Message) {
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
    async fn ready(&self, ctx: poise::serenity_prelude::Context, ready: Ready) {
        info!("attempting to register slash command for bridgebot::bridge");
        //let _ = Command::create_global_command(&ctx, commands::bridge::register()).await;
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

// this data is used everywhere, somehow
struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    debug!("[-] hello, world, from Rust BridgeBot.");
    debug!("[-] loading config from ENV...");
    dotenv().ok();
    debug!("[+] config loaded!");

    // perform migrations
    run_migrations();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    debug!("[-] building Framework object...");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![bridge()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

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
    ctx: &poise::serenity_prelude::Context,
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
