pub async fn say_text(ctx: crate::Context<'_>, msg: impl Into<String>) {
    if let Err(err) = ctx
        .send(
            poise::CreateReply::default()
                .embed(serenity::all::CreateEmbed::default().description(msg)),
        )
        .await
    {
        eprintln!("{:?}", err);
    }
}

pub async fn say_error(ctx: crate::Context<'_>, msg: impl Into<String>) {
    say_text_with_color(ctx, msg, serenity::all::colours::branding::RED).await;
}

async fn say_text_with_color(
    ctx: crate::Context<'_>,
    msg: impl Into<String>,
    color: serenity::all::Color,
) {
    if let Err(err) = ctx
        .send(
            poise::CreateReply::default().embed(
                serenity::all::CreateEmbed::default()
                    .color(color)
                    .description(msg),
            ),
        )
        .await
    {
        eprintln!("{:?}", err);
    }
}
