use emojis;
use poise::serenity_prelude as serenity;
use std::num::NonZeroU64;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
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

    // Get the current channel ID
    let channel1_o = ctx.channel_id();
    let channel1_name = channel1_o.name(ctx).await?;

    // Convert to Serenity's ChannelId type
    let channel2_o = serenity::ChannelId::from(NonZeroU64::new(channel2 as u64).unwrap());

    let guild_o = ctx.guild_id().unwrap();
    let guild_name = guild_o.name(ctx.cache()).unwrap();

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
                    let emoji = emojis::get_by_shortcode("white_check_mark").unwrap();
                    let emoji3 = emojis::get_by_shortcode("loud_sound").unwrap();
                    ctx.say(format!("{} Successfully registered `{}` => `{}`\n\nensure other direction is also registered; or do nothing for a one-way experience {}", emoji, channel1_o.get(), channel2_o.get(), emoji3)).await?;

                    let emoji2 = emojis::get_by_shortcode("warning").unwrap();
                    channel2_o.say(ctx, format!("{} bridge registration request from `{}` `#{}`\n\nrespond with `/bridge` and Channel ID `{}` to bridge back; or do nothing for a one-way experience {}", emoji2, guild_name, channel1_name, channel1_o, emoji3)).await?;
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
