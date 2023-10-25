use dotenv::dotenv;
use std::env;
use tokio;

use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::Client;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::QueryResult;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;

use serenity::model::prelude::ChannelId;

use rust_bridgebot::establish_connection;
use rust_bridgebot::models::*;
use rust_bridgebot::schema::channel_pairs::dsl::channel_pairs;

#[group]
#[commands(ping, getcurrentid, register)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("[-] message spotted by EventHandler");

        // fast-fail to prevent spamming / looping
        if msg.author.bot {
            return;
        }

        // let connection = &mut establish_connection();

        // let channel_id = msg.channel_id;

        // let results = channel_pairs
        //     .select((channel1, channel2)) // Select the columns you need
        //     .filter(channel1.eq(channel_id))
        //     .load::<(i64, i64)>(connection); // Change to the appropriate types
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("Bot is ready as {}!", ready.user.name);
    }
}

//fn get_channel_pairs(channel_id: i64) -> Result<Vec<ChannelPair>, Box<dyn std::error::Error>> {
fn get_channel_pairs(channel_id: i64) {
    let connection = &mut establish_connection();

    println!("[-] db query for channel id {}", channel_id);
    use rust_bridgebot::schema::channel_pairs::dsl::*;

    let results = channel_pairs
        .select((channel1, channel2)) // Select the columns you need
        .filter(channel1.eq(channel_id))
        .load::<(i64, i64)>(connection); // Change to the appropriate types

    for result in &results {
        println!("{:?}", result);
    }

    // let results = channel_pairs
    //     .select((channel1, channel2)) // Select the columns you need
    //     .filter(channel1.eq(channel_id))
    //     .load::<ChannelPair>(&mut connection)?;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[-] hello, world, from Rust BridgeBot.");
    println!("[-] loading config from ENV...");
    dotenv().ok();
    println!("[+] config loaded!");

    let connection = &mut establish_connection();
    //println!("{:?}", connection);

    let new_pair = ChannelPair {
        id: None,
        channel1: 123, // Replace with the actual ChannelID
        channel2: 456, // Replace with the actual ChannelID
    };

    let result = diesel::insert_into(channel_pairs::table())
        .values(&new_pair)
        .execute(connection);
    println!("{:?}", result);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP);

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

    get_channel_pairs(channel_id.into());

    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn getcurrentid(ctx: &Context, msg: &Message) -> CommandResult {
    println!("[-] current channel ID has been requested.");
    // Get the Channel ID of the message
    let channel_id = msg.channel_id;

    // Reply with the Channel ID
    msg.reply(
        ctx,
        format!("The `ChannelID` of this channel is: `{}`", channel_id),
    )
    .await?;

    Ok(())
}

#[command]
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
    let connection = &mut establish_connection();

    // Extract the text content of the message
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next(); // Skip the command (~register)

    // Get the second element (Channel ID)
    let channel_id_str = iter.next();

    // Declare channel2 outside the block
    let channel2: i64;

    // Get the second element (Channel ID)
    if let Some(channel_id_str) = channel_id_str {
        // Attempt to parse the text as an i64 (replace with the actual parsing logic)
        channel2 = match channel_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                msg.reply(ctx, "Invalid ChannelID format").await?;
                return Ok(());
            }
        };
    } else {
        msg.reply(ctx, "Invalid ~register command format").await?;
        return Ok(());
    }

    // Get the ID of the channel where the message was sent
    let channel1 = *msg.channel_id.as_u64() as i64;

    // make sure we can see the channel target...
    match ctx.http.get_channel(channel2 as u64).await {
        Ok(channel) => {
            println!("true");
            // Create a new ChannelPair instance
            let new_pair = ChannelPair {
                id: None, // Omitting the id because it's auto-incremented
                channel1,
                channel2,
            };

            use rust_bridgebot::schema::channel_pairs; // Replace with the actual module path to your schema

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
