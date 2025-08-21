use crate::utils::{
    macros::{msg_err, reg_err},
    musics::{TrackMeta, get_gid_and_match_vcid},
};

#[poise::command(slash_command, guild_only)]
pub async fn now_playing(ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let (gid, _) = reg_err!(ctx, get_gid_and_match_vcid(ctx).await);
    let maybe_call_lock = ctx.data().songbird.clone().get(gid);
    let call_lock = reg_err!(
        ctx,
        maybe_call_lock.ok_or(anyhow::anyhow!("Not connected to any channel"))
    );
    let call = call_lock.lock().await;
    let song = reg_err!(
        ctx,
        call.queue().current().ok_or(anyhow::anyhow!("No song"))
    );

    msg_err!(
        ctx.send(song.data::<TrackMeta>().to_embed(ctx, "Now playing"))
            .await
    );
    Ok(())
}
