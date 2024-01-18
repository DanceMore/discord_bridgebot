use dotenv::dotenv;
use std::env;
use std::num::NonZeroU64;
use tokio;

use serenity::all::standard::Configuration;

use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::StandardFramework;
use serenity::model::gateway::Ready;
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
use discord_bridgebot::models::*;

extern crate env_logger;
#[macro_use]
extern crate log;

#[group]
#[commands(ping, getcurrentid, register)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
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

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Bot is ready as {}!", ready.user.name);
    }
}

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

    let framework = StandardFramework::new().group(&GENERAL_GROUP);

    framework.configure(Configuration::new().with_whitespace(true).prefix("!"));

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

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("[-] someone pinged me, ponging...");
    let channel_id = msg.channel_id;

    let _ = get_channel_pairs(channel_id.into());

    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
async fn getcurrentid(ctx: &Context, msg: &Message) -> CommandResult {
    println!("[-] current channel ID has been requested.");
    let channel_id = msg.channel_id;

    msg.reply(
        ctx,
        format!("The `ChannelID` of this channel is: `{}`", channel_id),
    )
    .await?;

    Ok(())
}

#[command]
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
    // it seems like limiting the scope of the imports is important?
    // I can definitely break code with name-conflicts by putting this at the top....
    use discord_bridgebot::schema::channel_pairs;
    let connection = &mut establish_connection();

    // Extract the text content of the message
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next(); // Skip the command (!register)

    // Get the second element (Channel ID)
    let channel_id_str = iter.next();

    // Declare channel2 outside the block
    let channel2: i64;

    // check that we have a Channel ID
    if let Some(channel_id_str) = channel_id_str {
        // Attempt to parse the text as an i64
        channel2 = match channel_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                msg.reply(ctx, "Invalid ChannelID format").await?;
                return Ok(());
            }
        };
    } else {
        msg.reply(ctx, "Invalid register command format").await?;
        return Ok(());
    }

    // Get the ID of the channel where the message was sent
    let channel1 = msg.channel_id;

    let channel2_o =
        serenity::model::id::ChannelId::from(NonZeroU64::new(channel2 as u64).unwrap());

    // make sure we can see the channel target...
    match ctx.http.get_channel(channel2_o).await {
        Ok(_channel) => {
            // alright, since we're here, let's create and save the channel to datbase.
            let new_pair = ChannelPair {
                id: None, // Omitting the id because it's auto-incremented
                channel1: channel1.into(),
                channel2: channel2,
            };

            let result = diesel::insert_into(channel_pairs::table)
                .values(&new_pair)
                .execute(connection);

            match result {
                Ok(_) => {
                    msg.reply(ctx, "Registration successful").await?;
                }
                Err(_) => {
                    msg.reply(ctx, "Error registering the ChannelID").await?;
                }
            }
        }
        Err(_) => {
            msg.reply(
                ctx,
                format!("I don't think I can see ChannelID `{}`", channel2),
            )
            .await?;
        }
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
