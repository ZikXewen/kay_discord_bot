use serenity::all::Mentionable;
use songbird::input::Compose;

use crate::utils::{msg_err, reg_err, say_error, say_text};

pub fn all_commands() -> Vec<crate::Command> {
    vec![play(), stop(), skip()]
}

#[poise::command(slash_command, guild_only)]
async fn play(
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
    let res: Vec<(String, String, u64)> = res
        .filter_map(|meta| Some((meta.title?, meta.source_url?, meta.duration?.as_secs())))
        .collect();
    let desc: Vec<_> = res
        .iter()
        .enumerate()
        .map(|(i, (title, url, secs))| {
            format!(
                "{}: [**{}**]({}) - {:02}:{:02}",
                i + 1,
                title,
                url,
                secs / 60,
                secs % 60
            )
        })
        .collect();
    let options: Vec<_> = res
        .iter()
        .enumerate()
        .map(|(i, (title, url, _))| {
            let mut label = format!("{}: {}", i + 1, title);
            let len = (0..=100).rfind(|&i| label.is_char_boundary(i)).unwrap_or(0);
            label.truncate(len);
            serenity::all::CreateSelectMenuOption::new(label, url)
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

#[poise::command(slash_command, guild_only, aliases("leave"))]
async fn stop(ctx: crate::Context<'_>) -> anyhow::Result<()> {
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

#[poise::command(slash_command, guild_only)]
async fn skip(ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let (gid, _) = reg_err!(ctx, get_gid_and_match_vcid(ctx).await);
    let maybe_call_lock = ctx.data().songbird.clone().get(gid);
    let call_lock = reg_err!(
        ctx,
        maybe_call_lock.ok_or(anyhow::anyhow!("Not connected to any channel"))
    );
    let call = call_lock.lock().await;

    reg_err!(ctx, call.queue().skip(), "Failed to skip");
    say_text(ctx, "Skipped").await;
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

fn get_gid_and_user_vcid(
    ctx: crate::Context<'_>,
) -> anyhow::Result<(serenity::all::GuildId, serenity::all::ChannelId)> {
    let guild = ctx
        .guild()
        .ok_or(anyhow::anyhow!("No guild. How did this happen?"))?;
    let gid = guild.id.clone();
    let vcid = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|state| state.channel_id)
        .ok_or(anyhow::anyhow!("Join a voice channel to use this command"))?
        .clone();
    Ok((gid, vcid))
}

async fn get_gid_and_match_vcid(
    ctx: crate::Context<'_>,
) -> anyhow::Result<(serenity::all::GuildId, serenity::all::ChannelId)> {
    let (gid, ucid) = get_gid_and_user_vcid(ctx)?;
    let bcid: serenity::all::ChannelId = ctx
        .data()
        .songbird
        .clone()
        .get(gid)
        .ok_or(anyhow::anyhow!("Not connected to any channel"))?
        .lock()
        .await
        .current_connection()
        .and_then(|conn| conn.channel_id)
        .ok_or(anyhow::anyhow!("Not connected to any channel"))?
        .0
        .into();
    if ucid != bcid {
        anyhow::bail!("Join my voice channel to use this command");
    }
    Ok((gid, ucid))
}

async fn add_track(
    ctx: crate::Context<'_>,
    call_lock: std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
    url: impl Into<String>,
) -> anyhow::Result<()> {
    let mut call = call_lock.lock().await;
    let mut src = songbird::input::YoutubeDl::new(ctx.data().http.clone(), url.into());
    let (title, url, duration, thumb) = src
        .aux_metadata()
        .await
        .ok()
        .and_then(|m| Some((m.title?, m.source_url?, m.duration?.as_secs(), m.thumbnail?)))
        .ok_or(anyhow::anyhow!("Failed to fetch track info"))?;
    call.enqueue_input(src.into()).await;
    let msg = poise::CreateReply::default().embed(
        serenity::all::CreateEmbed::default()
            .title("Added Track")
            .description(format!(
                "[**{}**]({}) - {:02}:{:02}",
                title,
                url,
                duration / 60,
                duration % 60
            ))
            .footer(
                serenity::all::CreateEmbedFooter::new(ctx.author().display_name())
                    .icon_url(ctx.author().static_face()),
            )
            .image(thumb)
            .color(serenity::all::colours::branding::GREEN),
    );
    if let Err(err) = ctx.send(msg).await {
        eprintln!("{:?}", err);
    }
    Ok(())
}
