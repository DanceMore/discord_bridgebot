use serenity::all::Message;

use discord_bridgebot::establish_connection;
use discord_bridgebot::models::*;
use std::num::NonZeroU64;

use diesel::RunQueryDsl;

use crate::Data;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
pub async fn bridge(
    ctx: Context<'_>,
    #[description = "the target channel for the bridge"] channel_id: String,
) -> Result<(), Error> {
    println!("[-] inside bridge registration");

    use discord_bridgebot::schema::channel_pairs;
    let connection = &mut establish_connection();

    // Attempt to parse the provided channel ID
    let channel2 = match channel_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            ctx.say("Invalid ChannelID format").await?;
            return Ok(());
        }
    };

    // Get the current channel ID
    let channel1 = ctx.channel_id();

    // Convert to Serenity's ChannelId type
    let channel2_o = serenity::ChannelId::from(NonZeroU64::new(channel2 as u64).unwrap());

    //// Check if we can access the target channel
    //match ctx.http().get_channel(channel2_o).await {
    //    Ok(_) => {
    //        // Create and save the new channel pair to the database
    //        let new_pair = ChannelPair {
    //            id: None,
    //            channel1: channel1.into(),
    //            channel2: channel2,
    //        };

    //        match diesel::insert_into(channel_pairs::table)
    //            .values(&new_pair)
    //            .execute(connection) {
    //            Ok(_) => ctx.say("Registration successful").await?,
    //            Err(_) => ctx.say("Error registering the ChannelID").await?,
    //        }
    //    },
    //    Err(_) => {
    //        ctx.say(format!("I don't think I can see ChannelID `{}`", channel2))
    //            .await?;
    //    }
    //}

    Ok(())
}
