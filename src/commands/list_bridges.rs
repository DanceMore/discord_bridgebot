use emojis;

use diesel::{BoolExpressionMethods, RunQueryDsl};
use diesel::{ExpressionMethods, QueryDsl};

use discord_bridgebot::checks::is_guild_owner;
#[allow(unused_imports)]
use discord_bridgebot::data::{Context, Data, Error};
use discord_bridgebot::establish_connection;
use discord_bridgebot::models::ChannelPair;
use discord_bridgebot::schema::channel_pairs::dsl::*;

#[poise::command(slash_command, ephemeral=true, description_localized("en-US", "list all bridges related to the current channel"))]
pub async fn list_bridges(ctx: Context<'_>) -> Result<(), Error> {
    debug!("[-] inside list_bridges");

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

    let emoji_bridge = emojis::get_by_shortcode("bridge_at_night").unwrap();
    let mut message = format!(
        "{} Bridges for Channel ID `{}`:\n",
        emoji_bridge, current_channel_id
    );

    for pair in &results {
        let chan1 = if pair.channel1 == current_channel_id {
            format!("`{}`!", pair.channel1)
        } else {
            format!("`{}`", pair.channel1)
        };

        let chan2 = if pair.channel2 == current_channel_id {
            format!("`{}`!", pair.channel2)
        } else {
            format!("`{}`", pair.channel2)
        };

        message.push_str(&format!("- Channel ID {} => Channel ID {}\n", chan1, chan2));
    }

    // Discord messages are capped to 2000 characters.
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
