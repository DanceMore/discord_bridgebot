use emojis;

use diesel::{BoolExpressionMethods, RunQueryDsl};
use diesel::{ExpressionMethods, QueryDsl};

use discord_bridgebot::establish_connection;
use discord_bridgebot::checks::is_guild_owner;
use discord_bridgebot::models::ChannelPair;
use discord_bridgebot::schema::channel_pairs::dsl::*;
#[allow(unused_imports)]
use discord_bridgebot::data::{Context, Data, Error};

#[poise::command(slash_command, owners_only, check=is_guild_owner, description_localized("en-US", "delete all registrations related to current channel"))]
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

    for pair in results {
        // Delete each pair
        match diesel::delete(channel_pairs.filter(id.eq(pair.id))).execute(connection) {
            Ok(_) => {
                let emoji = emojis::get_by_shortcode("boom").unwrap();
                ctx.say(format!(
                    "{} Successfully unbridged Channel ID `{}` from Channel ID `{}`",
                    emoji, pair.channel1, pair.channel2
                ))
                .await?;
            }
            Err(_) => {
                let emoji = emojis::get_by_shortcode("x").unwrap();
                error!("[!] error deleting ChannelPair Object ID {}", pair.id);
                ctx.say(format!(
                    "{} Error unbridging the Channel ID `{}` from Channel ID `{}`, notify an administrator.",
                    emoji, pair.channel1, pair.channel2
                ))
                .await?;
            }
        }
    }
    //        },
    //        Ok(_) => {
    //            let emoji = emojis::get_by_shortcode("thinking").unwrap();
    //            ctx.say(format!("{} No bridges found for this channel", emoji)).await?;
    //        },
    //        Err(_) => {
    //            let emoji = emojis::get_by_shortcode("thinking").unwrap();
    //            ctx.say(format!("{} Error accessing the bridge data", emoji)).await?;
    //        }
    //    };

    Ok(())
}
