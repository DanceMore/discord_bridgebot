use emojis;
use poise::serenity_prelude as serenity;
use std::num::NonZeroU64;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;

use discord_bridgebot::checks::is_guild_owner;
use discord_bridgebot::establish_connection;
use discord_bridgebot::models::*;
use discord_bridgebot::schema::channel_pairs;

#[allow(unused_imports)]
use discord_bridgebot::data::{Context, Data, Error};

#[poise::command(slash_command, guild_only, check=is_guild_owner, description_localized("en-US", "bridge messages from here to there..."))]
pub async fn bridge(
    ctx: Context<'_>,
    #[description = "the target channel for the bridge"] channel_id: String,
) -> Result<(), Error> {
    debug!("[-] inside bridge registration");

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

    // Get the current channel ID and current guild
    let channel1_o = ctx.channel_id();
    let channel1_name = channel1_o.name(ctx).await?;

    let guild_o = ctx.guild_id().unwrap();
    let guild_name = guild_o.name(ctx.cache()).unwrap();

    // get info about channel2 and guild2
    let channel2_o = serenity::ChannelId::from(NonZeroU64::new(channel2 as u64).unwrap());
    let channel2_name = channel2_o.name(ctx).await?;
    let channel2_data = ctx.http().get_channel(channel2_o).await?;
    let guild2 = channel2_data.guild().unwrap();
    let guild2_o = guild2.guild_id;
    let guild2_name = guild2_o.name(ctx.cache()).unwrap();

    // Check if trying to bridge the same channel
    if channel1_o == channel2_o {
        let emoji = emojis::get_by_shortcode("no_entry").unwrap();
        ctx.say(format!(
            "{} You cannot bridge a channel with itself! {}",
            emoji, emoji
        ))
        .await?;
        return Ok(());
    }

    // Check if we can access the target channel
    match ctx.http().get_channel(channel2_o).await {
        Ok(_) => {
            // Create and save the new channel pair to the database
            let new_pair = InsertableChannelPair {
                id: None,
                channel1: channel1_o.into(),
                channel2: channel2,
            };

            match diesel::insert_into(channel_pairs::table)
                .values(&new_pair)
                .execute(connection)
            {
                Ok(_) => {
                    info!(
                        "[+] new ChannelPair registration for ChannelID {}",
                        channel1_o
                    );
                    let emoji1 = emojis::get_by_shortcode("white_check_mark").unwrap();
                    let emoji2 = emojis::get_by_shortcode("warning").unwrap();
                    let emoji3 = emojis::get_by_shortcode("loud_sound").unwrap();
                    let emoji_tada = emojis::get_by_shortcode("tada").unwrap();

                    let reverse_exists = channel_pairs::table
                        .filter(channel_pairs::channel1.eq(channel2))
                        .filter(channel_pairs::channel2.eq(channel1_o.get() as i64))
                        .first::<ChannelPair>(connection)
                        .optional()
                        .map(|opt| opt.is_some())
                        .unwrap_or(false);

                    // reply channel 1 aka originator
                    let followup1_msg: String = if reverse_exists {
                        format!("{} Successfully registered `{}` => `{}`\n{} Notification sent to `{}` `#{}`\n\n{} Messages should now flow both directions {}", emoji1, channel1_o.get(), channel2_o.get(), emoji1, guild2_name, channel2_name, emoji_tada, emoji_tada)
                    } else {
                        format!("{} Successfully registered `{}` => `{}`\n{} Notification sent to `{}` `#{}`\n\nEnsure other direction is also registered; or do nothing for a one-way experience {}", emoji1, channel1_o.get(), channel2_o.get(), emoji1, guild2_name, channel2_name, emoji3)
                    };
                    ctx.say(followup1_msg).await?;

                    // reply channel 2 aka target
                    let followup2_msg = if reverse_exists {
                        format!("{} Bridge fully established with `{}` `#{}`.\n\n{} Messages should now flow both directions {}", emoji1, guild_name, channel1_name, emoji_tada, emoji_tada)
                    } else {
                        format!("{} Bridge registration request from `{}` `#{}`\n\nRespond with `/bridge` and Channel ID `{}` to bridge back; or do nothing for a one-way experience {}", emoji2, guild_name, channel1_name, channel1_o, emoji3)
                    };

                    channel2_o.say(ctx, followup2_msg).await?;

                    return Ok(());
                }
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, ref info)) => {
                    // Optional: use `info.message()` or `info.details()` for better diagnostics
                    warn!(
                        "[!] Unique constraint violation while registering channel pair: {}",
                        info.details().unwrap_or("no details")
                    );
                    let emoji = emojis::get_by_shortcode("warning").unwrap();
                    ctx.say(format!(
                        "{} This channel pair is already registered. Check the other side? ",
                        emoji
                    ))
                    .await?;

                    return Ok(());
                }
                Err(_) => {
                    error!("[!] INSERT failure into ChannelPairs table");
                    let emoji = emojis::get_by_shortcode("x").unwrap();
                    ctx.say(format!(
                        "{} Error registering the Channel ID, notify an administrator. {}",
                        emoji, emoji
                    ))
                    .await?;
                    return Ok(());
                }
            }
        }
        Err(_) => {
            let emoji = emojis::get_by_shortcode("thinking").unwrap();
            ctx.say(format!(
                "{} I don't think I can see Channel ID `{}`",
                emoji, channel2
            ))
            .await?
        }
    };

    Ok(())
}
