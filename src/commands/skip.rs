use crate::utils::{
    macros::reg_err,
    musics::get_gid_and_match_vcid,
    replies::{say_error, say_text},
};

#[poise::command(slash_command, guild_only)]
pub async fn skip(ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let (gid, _) = reg_err!(ctx, get_gid_and_match_vcid(ctx).await);
    let maybe_call_lock = ctx.data().songbird.clone().get(gid);
    let call_lock = reg_err!(
        ctx,
        maybe_call_lock.ok_or(anyhow::anyhow!("Not connected to any channel"))
    );
    let call = call_lock.lock().await;

    if call.queue().is_empty() {
        say_error(ctx, "Nothing to skip").await;
        return Ok(());
    }

    reg_err!(ctx, call.queue().skip(), "Failed to skip");
    say_text(ctx, "Skipped").await;
    Ok(())
}
