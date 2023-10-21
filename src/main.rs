use dotenv::dotenv;

use serenity::client::{Context, EventHandler};
use serenity::framework::standard::StandardFramework;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{group, command};
use serenity::Client;
use serenity::http::CacheHttp;
use serenity::model::gateway::Ready;


use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;

use std::env;

use tokio;

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
async fn main() {
	println!("[-] hello, world, from Rust BridgeBot.");
	println!("[-] loading config from ENV...");
    dotenv().ok();
	println!("[+] config loaded!");


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
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
