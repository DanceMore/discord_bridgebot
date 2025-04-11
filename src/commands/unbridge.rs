use emojis;
use std::num::NonZeroU64;
use poise::serenity_prelude as serenity;

use discord_bridgebot::establish_connection;
use discord_bridgebot::checks::is_guild_owner;
use discord_bridgebot::models::ChannelPair;
use discord_bridgebot::schema::channel_pairs::dsl::channel_pairs;
use discord_bridgebot::schema::channel_pairs::*;
#[allow(unused_imports)]
use discord_bridgebot::data::{Context, Data, Error};

use diesel::BoolExpressionMethods;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;

#[poise::command(slash_command, guild_only, check=is_guild_owner, description_localized("en-US", "remove a specific bridge"))]
pub async fn unbridge(
    ctx: Context<'_>,
    // we MUST leave this as `channel_id` because it shows up client-side...
    #[description = "the target channel for the unbridge"] channel_id: String,
) -> Result<(), Error> {
    debug!("[-] inside unbridge registration");

    let connection = &mut establish_connection();

    // Attempt to parse the provided channel ID
    let channel2_id_from_str: i64 = match channel_id.parse::<i64>() {
        Ok(channel_id) => channel_id,
        Err(_) => {
            let emoji = emojis::get_by_shortcode("warning").unwrap();
            ctx.say(format!("{} Invalid ChannelID format; expecting `String` containing `Integer` `ChannelID` {}", emoji, emoji)).await?;
            return Ok(());
        }
    };

    // Get the current channel ID
    let channel1_o = ctx.channel_id();
    let channel1_oid = channel1_o.get() as i64;

    // Convert to Serenity's ChannelId type
    let channel2_o =
        serenity::ChannelId::from(NonZeroU64::new(channel2_id_from_str as u64).unwrap());
    let channel2_oid = channel2_o.get() as i64;

    // Check if trying to unbridge the same channel
    // TODO: evaluate if this is needed
    if channel1_o == channel2_o {
        let emoji = emojis::get_by_shortcode("no_entry").unwrap();
        ctx.say(format!(
            "{} You cannot unbridge a channel with itself! {}",
            emoji, emoji
        ))
        .await?;
        return Ok(());
    }

    let results: Vec<ChannelPair> = channel_pairs
        .filter(channel1.eq(channel1_oid).and(channel2.eq(channel2_oid)))
        .load(connection)
        .expect("error fetching");

    // Attempt to find all pairs where channel1 is involved
    debug!("[!] {} total results", results.len());
    for each in &results {
        debug!("[.] {:?}", each);
    }

    if results.is_empty() {
        let emoji = emojis::get_by_shortcode("interrobang").unwrap();
        ctx.say(format!(
            "{} No bridges found for `this` current channelID `{}` {}",
            emoji, channel1_oid, emoji
        ))
        .await?;
        return Ok(());
    }

    for pair in results {
        // If the pair exists, delete it
        match diesel::delete(channel_pairs.filter(id.eq(pair.id))).execute(connection) {
            Ok(_) => {
                let emoji = emojis::get_by_shortcode("boom").unwrap();
                ctx.say(format!(
                    "{} Successfully unbridged the channels {}",
                    emoji, emoji
                ))
                .await?;
            }
            Err(_) => {
                error!("[!] DELETE failure on ChannelPairs table");
                let emoji = emojis::get_by_shortcode("x").unwrap();
                ctx.say(format!(
                    "{} Error deleting the ChannelID, notify an administrator. {}",
                    emoji, emoji
                ))
                .await?;
                return Ok(());
            }
        }
    }

    Ok(())
}
