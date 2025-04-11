use crate::Data;
//struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// because of guild_only, we should NEVER enter here as a DM
// therefore no DM handling logic ;)
pub async fn is_guild_owner(ctx: Context<'_>) -> Result<bool, Error> {
    info!("[?] checking for Guild Ownership...");
    // Ensure the command is being run in a guild (not a DM)A
    if let Some(guild_id) = ctx.guild() {
        // Get the guild's owner ID
        let owner_id = guild_id.owner_id;

        // Check if the user executing the command is the guild owner
        if owner_id == ctx.author().id {
            return Ok(true);
        }
    }
    // Return false if the user is not the guild owner or it's a DM
    warn!("[!] user is NOT a guild owner....");
    let emoji = emojis::get_by_shortcode("x").unwrap();
    let _ = ctx
        .say(format!(
            "{} you are not a Guild Owner, please stop. {}",
            emoji, emoji
        ))
        .await;
    Ok(false)
}
