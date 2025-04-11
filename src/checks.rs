use crate::Data;
//struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn is_guild_owner(ctx: Context<'_>) -> Result<bool, Error> {
    // Ensure the command is being run in a guild (not a DM)
    //if let Some(guild_id) = ctx.guild_id() {
    //    // Get the guild's owner ID
    //    let owner_id = guild_id.owner_id().await?;

    //    // Check if the user executing the command is the guild owner
    //    if owner_id == ctx.author().id {
    //        return Ok(true);
    //    }
    //}
    // Return false if the user is not the guild owner or it's a DM
    Ok(false)
}
