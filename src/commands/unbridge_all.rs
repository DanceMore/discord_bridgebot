//use diesel::query_dsl::methods::FilterDsl;
//use diesel::SelectableHelper;
//use discord_bridgebot::schema::channel_pairs::channel1;
//use serenity::model::id::ChannelId;

use emojis;

use discord_bridgebot::{establish_connection, models::ChannelPair};
//use discord_bridgebot::models::*;
//use std::num::NonZeroU64;

use diesel::{query_dsl::methods::LimitDsl, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel::{BoolExpressionMethods, RunQueryDsl};

use crate::Data;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
pub async fn unbridge_all(ctx: Context<'_>) -> Result<(), Error> {
    debug!("[-] inside unbridge_all registration");

    //use discord_bridgebot::schema::channel_pairs;
    use discord_bridgebot::schema::channel_pairs::dsl::*;
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
