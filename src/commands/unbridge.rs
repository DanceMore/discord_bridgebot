use emojis;

use discord_bridgebot::establish_connection;
use std::num::NonZeroU64;


use crate::Data;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[poise::command(slash_command, guild_only)]
pub async fn unbridge(
    ctx: Context<'_>,
    #[description = "the target channel for the unbridge"] channel_id_str: String,
) -> Result<(), Error> {
    debug!("[-] inside unbridge registration");

    let connection = &mut establish_connection();

    // Attempt to parse the provided channel ID
    let channel2_id_from_str: i64 = match channel_id_str.parse::<i64>() {
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
    let channel2_o = serenity::ChannelId::from(NonZeroU64::new(channel2_id_from_str as u64).unwrap());
    let channel2_oid = channel2_o.get() as i64;

    // Check if trying to unbridge the same channel
    // TODO: evaluate if this is needed
    if channel1_o == channel2_o {
        let emoji = emojis::get_by_shortcode("no_entry").unwrap();
        ctx.say(format!("{} You cannot unbridge a channel with itself! {}", emoji, emoji)).await?;
        return Ok(());
    }

   // let results: Vec<ChannelPair> = channel_pairs
   // .filter(
   //     channel1
   //         .eq(channel1_oid)
   //         .and(channel2.eq(channel2_oid)),
   // )
   // .load(connection)
   // .expect("error fetching");

// Attempt to find all pairs where channel1 is involved
//println!("{} total results", results.len());
//for each in &results {
//    println!("[-] {:?}", each);
//}
//
//if results.is_empty() {
//    let emoji = emojis::get_by_shortcode("interrobang").unwrap();
//    ctx.say(format!(
//        "{} No bridges found for `this` current channelID `{}` {}",
//        emoji, current_channel_id, emoji
//    ))
//    .await?;
//    return Ok(());
//}


    // Attempt to find the pair in the database
    //let target_pair = diesel::select(channel_pairs::table)
    //    .filter(channel_pairs::channel1.eq(channel1.into()))
    //    .filter(channel_pairs::channel2.eq(channel2))
    //    .first::<ChannelPair>(connection);

    //match target_pair {
    //    Ok(pair) => {
    //        // If the pair exists, delete it
    //        match diesel::delete(channel_pairs::table.filter(channel_pairs::id.eq(pair.id.unwrap())))
    //            .execute(connection)
    //        {
    //            Ok(_) => {
    //                let emoji = emojis::get_by_shortcode("check_mark").unwrap();
    //                ctx.say(format!("{} Successfully unbridged the channels {}", emoji, emoji)).await?;
    //            }
    //            Err(_) => {
    //                let emoji = emojis::get_by_shortcode("error").unwrap();
    //                ctx.say(format!("{} Error unbridging the ChannelID", emoji)).await?;
    //            }
    //        }
    //    },
    //    Err(_) => {
    //        let emoji = emojis::get_by_shortcode("thinking").unwrap();
    //        ctx.say(format!("{} No bridge found for ChannelID `{}` {}", emoji, channel2, emoji)).await?;
    //    }
    //};

    Ok(())
}
