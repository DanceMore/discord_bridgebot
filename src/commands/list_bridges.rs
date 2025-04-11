use emojis;

use diesel::{BoolExpressionMethods, RunQueryDsl};
use diesel::{ExpressionMethods, QueryDsl};

use discord_bridgebot::checks::is_guild_owner;
#[allow(unused_imports)]
use discord_bridgebot::data::{Context, Data, Error};
use discord_bridgebot::establish_connection;
use discord_bridgebot::models::ChannelPair;
use discord_bridgebot::schema::channel_pairs::dsl::*;

#[poise::command(slash_command, check=is_guild_owner, description_localized("en-US", "list all bridges related to the current channel"))]
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

    for pair in results {
                ctx.say(format!(
                    "Channel ID `{}` => Channel ID `{}`",
                    pair.channel1, pair.channel2
                ))
                .await?;
    }

    Ok(())
}
