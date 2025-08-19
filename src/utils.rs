macro_rules! reg_err {
    ($ctx: ident, $x: expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                say_error($ctx, err.to_string()).await;
                return Ok(());
            }
        }
    };
    ($ctx: ident, $x: expr, $err: expr) => {
        match $x {
            Ok(val) => val,
            Err(_) => {
                say_error($ctx, $err).await;
                return Ok(());
            }
        }
    };
}

macro_rules! msg_err {
    ($x: expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{}", err);
                return Ok(());
            }
        }
    };
}

pub(crate) use msg_err;
pub(crate) use reg_err;

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
