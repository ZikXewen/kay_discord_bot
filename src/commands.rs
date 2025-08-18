use serenity::all::Mentionable;
use songbird::input::Compose;

type Command = poise::Command<crate::Data, anyhow::Error>;
type Context<'a> = poise::Context<'a, crate::Data, anyhow::Error>;

pub fn all_commands() -> Vec<Command> {
    vec![play(), stop()]
}

fn checked<T>(result: serenity::Result<T>) {
    if let Err(err) = result {
        eprintln!("{:?}", err);
    }
}

#[poise::command(slash_command, guild_only)]
async fn play(
    ctx: Context<'_>,
    #[description = "URL/Query to play"] input: String,
) -> anyhow::Result<()> {
    let (gid, maybe_vcid) = {
        let guild = ctx.guild().ok_or(anyhow::anyhow!("No guild"))?;
        let maybe_vcid = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|state| state.channel_id);
        (guild.id, maybe_vcid)
    };
    let vcid = match maybe_vcid {
        Some(c) => c,
        None => {
            checked(ctx.reply("Join a voice channel to use this command").await);
            return Ok(());
        }
    };

    let manager = ctx.data().songbird.clone();

    let maybe_call_lock = match manager.get(gid) {
        Some(cl) if cl.lock().await.current_connection().is_some() => Some(cl),
        _ => {
            checked(ctx.say(format!("Joining {}", vcid.mention())).await);
            manager.join(gid, vcid).await.ok()
        }
    };
    let call_lock = match maybe_call_lock {
        Some(cl) => cl,
        None => {
            checked(ctx.reply(format!("Failed to join the channel")).await);
            return Ok(());
        }
    };

    let mut call = call_lock.lock().await;
    if let Some(cid) = call
        .current_connection()
        .and_then(|conn| conn.channel_id)
        .map(|c| serenity::all::ChannelId::from(c.0))
        && cid != vcid
    {
        checked(
            ctx.reply(format!("Already connected to {}", cid.mention()))
                .await,
        );
        return Ok(());
    }

    let is_url = input.starts_with("https://") || input.starts_with("http://");
    let mut src = if is_url {
        songbird::input::YoutubeDl::new(ctx.data().http.clone(), input)
    } else {
        songbird::input::YoutubeDl::new_search(ctx.data().http.clone(), input)
    };
    let mut ret_str = String::from("Playing song");
    if let Ok(data) = src.aux_metadata().await {
        data.title
            .inspect(|title| ret_str.push_str(&format!(": {}", title)));
        data.artist
            .inspect(|artist| ret_str.push_str(&format!(" by {}", artist)));
        data.duration.inspect(|duration| {
            let seconds = duration.as_secs();
            ret_str.push_str(&format!(" - {}:{}", seconds / 60, seconds % 60));
        });
    }
    checked(ctx.reply(ret_str).await);

    // TODO: replace with queue system
    call.stop();
    call.play_input(src.into());

    Ok(())
}

#[poise::command(slash_command, guild_only)]
async fn stop(ctx: Context<'_>) -> anyhow::Result<()> {
    let (gid, maybe_vcid) = {
        let guild = ctx.guild().ok_or(anyhow::anyhow!("No guild"))?;
        let maybe_vcid = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|state| state.channel_id);
        (guild.id, maybe_vcid)
    };
    let vcid = match maybe_vcid {
        Some(c) => c,
        None => {
            checked(ctx.reply("Join a voice channel to use this command").await);
            return Ok(());
        }
    };

    let manager = ctx.data().songbird.clone();
    let call_lock = match manager.get(gid) {
        Some(cl) if cl.lock().await.current_connection().is_some() => cl,
        _ => {
            checked(ctx.reply("Nothing to stop").await);
            return Ok(());
        }
    };

    let mut call = call_lock.lock().await;
    let cid = call
        .current_connection()
        .and_then(|conn| conn.channel_id)
        .map(|c| serenity::all::ChannelId::from(c.0))
        .ok_or(anyhow::anyhow!("No channel"))?;

    if cid != vcid {
        checked(ctx.reply("Join my voice channel to use this command").await);
        return Ok(());
    }

    checked(
        ctx.reply(format!("Stopped and disconnected from {}", cid.mention()))
            .await,
    );
    call.leave().await?;
    Ok(())
}
