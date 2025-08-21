pub struct TrackMeta {
    pub title: String,
    pub url: String,
    pub thumb: String,
    pub secs: u64,
}

impl TrackMeta {
    pub fn to_embed(&self, ctx: crate::Context<'_>, title: &'static str) -> poise::CreateReply {
        poise::CreateReply::default().embed(
            serenity::all::CreateEmbed::default()
                .title(title)
                .description(self.to_string())
                .footer(
                    serenity::all::CreateEmbedFooter::new(ctx.author().display_name())
                        .icon_url(ctx.author().static_face()),
                )
                .image(&self.thumb)
                .color(serenity::all::colours::branding::GREEN),
        )
    }
}

impl TryFrom<songbird::input::AuxMetadata> for TrackMeta {
    type Error = anyhow::Error;
    fn try_from(m: songbird::input::AuxMetadata) -> Result<Self, Self::Error> {
        match (m.title, m.source_url, m.thumbnail, m.duration) {
            (Some(title), Some(url), Some(thumb), Some(duration)) => Ok(Self {
                title,
                url,
                thumb,
                secs: duration.as_secs(),
            }),
            _ => Err(anyhow::anyhow!("Failed to parse track data")),
        }
    }
}

impl std::fmt::Display for TrackMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[**{}**]({}) - {:02}:{:02}",
            self.title,
            self.url,
            self.secs / 60,
            self.secs % 60
        )
    }
}

pub fn get_gid_and_user_vcid(
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

pub async fn get_gid_and_match_vcid(
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
