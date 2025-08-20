use crate::utils::{
    macros::reg_err,
    musics::{TrackMeta, get_gid_and_match_vcid},
    replies::say_text,
};

#[poise::command(slash_command, guild_only)]
pub async fn queue(ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let (gid, _) = reg_err!(ctx, get_gid_and_match_vcid(ctx).await);
    let maybe_call_lock = ctx.data().songbird.clone().get(gid);
    let call_lock = reg_err!(
        ctx,
        maybe_call_lock.ok_or(anyhow::anyhow!("Not connected to any channel"))
    );
    let call = call_lock.lock().await;

    if call.queue().is_empty() {
        say_text(ctx, "Queue is empty").await;
        return Ok(());
    }

    let queue = call
        .queue()
        .current_queue()
        .into_iter()
        .enumerate()
        .map(|(i, track)| format!("{}: {}", i + 1, track.data::<TrackMeta>().clone()))
        .collect::<Vec<_>>()
        .join("\n");
    say_text(ctx, queue).await;
    Ok(())
}
