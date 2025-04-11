use emojis;

use diesel::{BoolExpressionMethods, RunQueryDsl};
use diesel::{ExpressionMethods, QueryDsl};

use discord_bridgebot::{establish_connection, models::ChannelPair};
use discord_bridgebot::schema::channel_pairs::dsl::*;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use discord_bridgebot::checks::is_guild_owner;

#[poise::command(slash_command, owners_only, check=is_guild_owner)]
pub async fn unbridge_all(ctx: Context<'_>) -> Result<(), Error> {
    debug!("[-] inside unbridge_all registration");

    //use discord_bridgebot::schema::channel_pairs;
    let connection = &mut establish_connection();

    // Get the current channel ID
    let current_channel = ctx.channel_id();
    let current_channel_id = current_channel.get() as i64;

    //let results = channel_pairs.filter(channel1.eq(current_channel_id)).(ChannelPair::as_select()).execute(connection).expect("error fetching");
    let results: Vec<ChannelPair> = channel_pairs
        .filter(
            channel1
                .eq(current_channel_id)
                .or(channel2.eq(current_channel_id)),
        )
        .load(connection)
        .expect("error fetching");

    // Attempt to find all pairs where channel1 is involved
    println!("{} total results", results.len());
    for each in &results {
        println!("[-] {:?}", each);
    }

    if results.is_empty() {
        let emoji = emojis::get_by_shortcode("interrobang").unwrap();
        ctx.say(format!(
            "{} No bridges found for `this` current channelID `{}` {}",
            emoji, current_channel_id, emoji
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
                    "{} Successfully unbridged channel `{}` from channel `{}`",
                    emoji, pair.channel1, pair.channel2
                ))
                .await?;
            }
            Err(_) => {
                let emoji = emojis::get_by_shortcode("x").unwrap();
                ctx.say(format!(
                    "{} Error unbridging the ChannelID `{}` `{}` `{}`",
                    emoji, pair.id, pair.channel1, pair.channel2
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
