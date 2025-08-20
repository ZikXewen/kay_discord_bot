use serenity::all::Mentionable;

use crate::utils::{macros::reg_err, musics::get_gid_and_match_vcid, replies::say_text};

#[poise::command(slash_command, guild_only, aliases("leave"))]
pub async fn stop(ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let (gid, vcid) = reg_err!(ctx, get_gid_and_match_vcid(ctx).await);
    let maybe_call_lock = ctx.data().songbird.clone().get(gid);
    let call_lock = reg_err!(
        ctx,
        maybe_call_lock.ok_or(anyhow::anyhow!("Not connected to any channel"))
    );
    let mut call = call_lock.lock().await;

    call.queue().stop();
    reg_err!(ctx, call.leave().await, "Failed to leave the call.");
    say_text(
        ctx,
        format!("Stopped and disconnected from {}", vcid.mention()),
    )
    .await;
    Ok(())
}
