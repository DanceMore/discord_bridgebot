use serenity::model::id::ChannelId;

use emojis;

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
    debug!("[-] inside bridge registration");

    use discord_bridgebot::schema::channel_pairs;
    let connection = &mut establish_connection();

    // Attempt to parse the provided channel ID
    let channel2 = match channel_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            let emoji = emojis::get_by_shortcode("warning").unwrap();
            ctx.say(format!("{} Invalid ChannelID format; expecting `String` containing `Integer` `ChannelID` {}", emoji, emoji)).await?;
            return Ok(());
        }
    };

    // Get the current channel ID
    let channel1 = ctx.channel_id();

    // Convert to Serenity's ChannelId type
    let channel2_o = serenity::ChannelId::from(NonZeroU64::new(channel2 as u64).unwrap());

    // Check if trying to bridge the same channel
    if channel1 == channel2_o {
        let emoji = emojis::get_by_shortcode("no_entry").unwrap();
        ctx.say(format!("{} You cannot bridge a channel with itself! {}", emoji, emoji)).await?;
        return Ok(());
    }

    // Check if we can access the target channel
    match ctx.http().get_channel(channel2_o).await {
        Ok(_) => {
            // Create and save the new channel pair to the database
            let new_pair = InsertableChannelPair {
                id: None,
                channel1: channel1.into(),
                channel2: channel2,
            };

            match diesel::insert_into(channel_pairs::table)
                .values(&new_pair)
                .execute(connection) {
                Ok(_) => ctx.say("Registration successful").await?,
                Err(_) => ctx.say("Error registering the ChannelID").await?,
            }
        },
        Err(_) => {
            let emoji = emojis::get_by_shortcode("thinking").unwrap();
            ctx.say(format!("{} I don't think I can see ChannelID `{}` {}", emoji, channel2, emoji)).await?
        }
    };

    Ok(())
}
