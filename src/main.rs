use dotenv::dotenv;
use serenity::all::UserId;
use std::collections::HashSet;
use std::env;
use std::num::NonZeroU64;
use tokio;

use poise::async_trait;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ActivityData;
use poise::serenity_prelude::Client;
use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::GatewayIntents;
use poise::serenity_prelude::Message;
use serenity::all::Ready;

mod commands;
use crate::commands::bridge::bridge;
use crate::commands::list_bridges::list_bridges;
use crate::commands::unbridge::unbridge;
use crate::commands::unbridge_all::unbridge_all;

use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations() {
    let mut conn = establish_connection();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

use discord_bridgebot::data::Data;
use discord_bridgebot::establish_connection;

extern crate env_logger;
#[macro_use]
extern crate log;

// this data is used everywhere, somehow
//struct Data {} // User data, which is stored and accessible in all command invocations
//#[allow(dead_code)]
//type Error = Box<dyn std::error::Error + Send + Sync>;
//#[allow(dead_code)]
//type Context<'a> = poise::Context<'a, Data, Error>;

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
            commands: vec![bridge(), list_bridges(), unbridge(), unbridge_all()],
            owners: HashSet::from([UserId::new(898051927206674443)]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let emoji_bridge = emojis::get_by_shortcode("bridge_at_night").unwrap();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .status(serenity::OnlineStatus::Online)
        .activity(ActivityData::custom(format!(
            "{} building bridges between communities... {}",
            emoji_bridge, emoji_bridge
        )))
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

// the main MEAT
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

            info!("[!] mirroring a message {} times", pairs.len());
            for pair in &pairs {
                debug!(
                    "[.] will mirror \"{}: {}\" to channel id {}",
                    padded_author, padded_message, pair.1
                );
                mirror_message(&ctx, pair.1, author, message).await;
            }
        } else {
            eprintln!("[-] Error while querying the database.");
        }
    }

    // ready up, battle bus is here ...
    async fn ready(&self, _ctx: poise::serenity_prelude::Context, ready: Ready) {
        info!("[!] Bot is ready as {}!", ready.user.name);
    }
} // end EventHandler

// TODO: improve this??
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
