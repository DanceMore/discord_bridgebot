use emojis;

use diesel::{BoolExpressionMethods, RunQueryDsl};
use diesel::{ExpressionMethods, QueryDsl};

use discord_bridgebot::checks::is_guild_owner;
#[allow(unused_imports)]
use discord_bridgebot::data::{Context, Data, Error};
use discord_bridgebot::establish_connection;
use discord_bridgebot::models::ChannelPair;
use discord_bridgebot::schema::channel_pairs::dsl::*;

use std::num::NonZeroU64;
use poise::serenity_prelude::ChannelId;

#[poise::command(
    slash_command,
    check = is_guild_owner,
    description_localized("en-US", "delete all registrations related to current channel")
)]
pub async fn unbridge_all(ctx: Context<'_>) -> Result<(), Error> {
    debug!("[-] inside unbridge_all");

    let connection = &mut establish_connection();

    // Get the current channel ID
    let current_channel = ctx.channel_id();
    let current_channel_id = current_channel.get() as i64;

    // fetch ChannelPairs where EITHER column matches the current channel
    // ie: all pairs where channel1 is involved
    let results: Vec<ChannelPair> = channel_pairs
        .filter(
            channel1
                .eq(current_channel_id)
                .or(channel2.eq(current_channel_id)),
        )
        .load(connection)
        .expect("error fetching");

    // debug logging :snore:
    debug!("[+] {} total results", results.len());
    for each in &results {
        debug!("[.] will unbridge => {:?}", each);
    }

    // announce no work to do
    if results.is_empty() {
        let emoji = emojis::get_by_shortcode("interrobang").unwrap();
        ctx.say(format!(
            "{} No bridges found for `this` current Channel ID `{}`",
            emoji, current_channel_id
        ))
        .await?;
        return Ok(());
    }

    let mut message = format!(
        "Attempting to unbridge all connections related to Channel ID `{}`:\nNotifications will be sent to the other side.\n\n",
        current_channel_id
    );

    for pair in results {
        // this logic is ass but it works
        let mut other_channel = ChannelId::from(NonZeroU64::new(pair.channel2 as u64).unwrap());
        let chan1 = if pair.channel1 == current_channel_id {
             other_channel = ChannelId::from(NonZeroU64::new(pair.channel1 as u64).unwrap());
             format!("`{}`!", pair.channel1)
        } else {
            format!("`{}`", pair.channel1)
        };

        let chan2 = if pair.channel2 == current_channel_id {
            format!("`{}`!", pair.channel2)
        } else {
            other_channel = ChannelId::from(NonZeroU64::new(pair.channel2 as u64).unwrap());
            format!("`{}`", pair.channel2)
        };

        match diesel::delete(channel_pairs.filter(id.eq(pair.id))).execute(connection) {
            Ok(_) => {
                let emoji = emojis::get_by_shortcode("boom").unwrap();
                let emoji_warning = emojis::get_by_shortcode("warning").unwrap();
                message.push_str(&format!(
                    "{} Successfully unbridged Channel ID {} => Channel ID {}\n",
                    emoji, chan1, chan2
                ));
                other_channel.say(ctx, format!("{} Owner of Channel ID `{}` initiated an unbridge.\nThis cannot be cancelled.", emoji_warning, current_channel_id)).await?;
            }
            Err(_) => {
                let emoji = emojis::get_by_shortcode("x").unwrap();
                error!("[!] error deleting ChannelPair Object ID {}", pair.id);
                message.push_str(&format!(
                    "{} Error unbridging Channel ID {} => Channel ID {}, notify an administrator.\n",
                    emoji, chan1, chan2
                ));
            }
        }
    }

    if message.len() > 2000 {
        for chunk in message.as_bytes().chunks(1900) {
            let chunk_str = String::from_utf8_lossy(chunk);
            ctx.say(chunk_str).await?;
        }
    } else {
        ctx.say(message).await?;
    }

    Ok(())
}