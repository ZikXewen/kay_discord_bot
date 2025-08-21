use serenity::all::Mentionable;
use songbird::input::Compose;

use crate::utils::{
    macros::{msg_err, reg_err},
    musics::{TrackMeta, get_gid_and_user_vcid},
    replies::say_error,
};

#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: crate::Context<'_>,
    #[description = "URL/Query to play"] track: String,
) -> anyhow::Result<()> {
    msg_err!(ctx.defer().await);
    let (gid, vcid) = reg_err!(ctx, get_gid_and_user_vcid(ctx));
    let call_lock = reg_err!(ctx, join_if_disconnected(ctx, gid, vcid).await);

    if track.starts_with("https://") || track.starts_with("http://") {
        reg_err!(ctx, add_track(ctx, call_lock, track).await);
        return Ok(());
    }

    let mut src = songbird::input::YoutubeDl::new_search(ctx.data().http.clone(), track);
    let res = reg_err!(ctx, src.search(Some(10)).await, "No songs found");
    let res: Vec<_> = res.filter_map(|m| TrackMeta::try_from(m).ok()).collect();
    let desc: Vec<_> = res
        .iter()
        .enumerate()
        .map(|(i, track)| format!("{}: {}", i + 1, track))
        .collect();
    let options: Vec<_> = res
        .into_iter()
        .enumerate()
        .map(|(i, track)| {
            let mut label = format!("{}: {}", i + 1, track.title);
            let len = (0..=100).rfind(|&i| label.is_char_boundary(i)).unwrap_or(0);
            label.truncate(len);
            serenity::all::CreateSelectMenuOption::new(label, track.url)
        })
        .collect();
    let msg = poise::CreateReply::default()
        .embed(
            serenity::all::CreateEmbed::default()
                .title("Search Results")
                .description(desc.join("\n")),
        )
        .components(vec![serenity::all::CreateActionRow::SelectMenu(
            serenity::all::CreateSelectMenu::new(
                "track",
                serenity::all::CreateSelectMenuKind::String { options },
            ),
        )])
        .ephemeral(true);

    let reply = msg_err!(ctx.send(msg).await);
    let msg = msg_err!(reply.message().await);
    let inter = match msg
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        Some(x) => x,
        None => {
            msg_err!(reply.delete(ctx).await);
            say_error(ctx, "Timed out").await;
            return Ok(());
        }
    };
    msg_err!(reply.delete(ctx).await);
    let url = match &inter.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => &values[0],
        _ => {
            say_error(ctx, "Unknown data kind. How did this happen?").await;
            return Ok(());
        }
    };

    reg_err!(ctx, add_track(ctx, call_lock, url).await);
    Ok(())
}

async fn join_if_disconnected(
    ctx: crate::Context<'_>,
    gid: serenity::all::GuildId,
    cid: serenity::all::ChannelId,
) -> anyhow::Result<std::sync::Arc<tokio::sync::Mutex<songbird::Call>>> {
    let manager = ctx.data().songbird.clone();
    let maybe_call_lock = match manager.get(gid) {
        Some(cl) if cl.lock().await.current_connection().is_some() => Some(cl),
        _ => manager.join(gid, cid).await.ok(),
    };
    let call_lock = maybe_call_lock.ok_or(anyhow::anyhow!("Failed to join the channel"))?;
    let maybe_current_cid = call_lock
        .lock()
        .await
        .current_connection()
        .and_then(|conn| conn.channel_id)
        .map(|c| serenity::all::ChannelId::from(c.0));
    match maybe_current_cid {
        Some(current_cid) if current_cid != cid => {
            anyhow::bail!(format!("Already connected to {}", current_cid.mention()))
        }
        None => anyhow::bail!("Failed to join the channel"),
        _ => Ok(call_lock),
    }
}

async fn add_track(
    ctx: crate::Context<'_>,
    call_lock: std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
    url: impl Into<String>,
) -> anyhow::Result<()> {
    let mut call = call_lock.lock().await;
    let mut src = songbird::input::YoutubeDl::new(ctx.data().http.clone(), url.into());
    let meta: TrackMeta = src
        .aux_metadata()
        .await
        .map_err(|_| anyhow::anyhow!("Failed to fetch track info"))?
        .try_into()?;
    let meta = std::sync::Arc::new(meta);
    let track = songbird::tracks::Track::new_with_data(src.into(), meta.clone());
    call.enqueue(track).await;
    msg_err!(ctx.send(meta.clone().to_embed(ctx, "Added Track")).await);
    Ok(())
}
