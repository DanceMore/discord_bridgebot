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
use diesel::RunQueryDsl;
use diesel::SelectableHelper;

use rust_bridgebot::establish_connection;
use rust_bridgebot::models::*;
use rust_bridgebot::schema::channel_pairs::dsl::channel_pairs;

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("I saw a message.");
        println!("{:?}", msg);
        // Your custom logic goes here to determine when to execute a command.

        // fast-fail to prevent spamming / looping
        if msg.author.bot {
            return;
        }

        if !msg.author.bot {
            // if let Err(why) = ctx.with_framework(|f| f.dispatch(ctx, &msg)) {
            //     println!("Error when dispatching command: {:?}", why);
            // }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("Bot is ready as {}!", ready.user.name);
    }
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

    diesel::insert_into(channel_pairs::table())
        .values(&new_pair)
        .execute(connection);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
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
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
