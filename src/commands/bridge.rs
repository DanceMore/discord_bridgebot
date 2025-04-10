use serenity::all::standard::macros::command;
use serenity::all::standard::CommandResult;
use serenity::all::CommandOptionType;
use serenity::all::Context;
use serenity::all::CreateCommand;
use serenity::all::CreateCommandOption;
use serenity::all::Message;

use discord_bridgebot::establish_connection;
use discord_bridgebot::models::*;
use std::num::NonZeroU64;

use diesel::RunQueryDsl;

#[command]
pub async fn run(ctx: &Context, msg: &Message) -> CommandResult {
    println!("[-] inside bridge registration");

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

pub fn register() -> CreateCommand {
    CreateCommand::new("bridge")
        .description("Bridge the current ChannelID to the $channel_id ChannelID")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "channel_id",
                "the target channel for the bridge, by ChannelID",
            )
            .required(true),
        )
}
