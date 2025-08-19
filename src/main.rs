mod commands;
mod utils;

use poise::serenity_prelude as serenity;

struct Data {
    http: reqwest::Client,
    songbird: std::sync::Arc<songbird::Songbird>,
}

type Command = poise::Command<Data, anyhow::Error>;
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN")?;
    let manager = songbird::Songbird::serenity();

    let manager_clone = manager.clone();
    let framework = poise::Framework::builder()
        .setup(move |ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    http: reqwest::Client::new(),
                    songbird: manager_clone,
                })
            })
        })
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            ..Default::default()
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged(); // TODO: review intents
    let mut client = serenity::Client::builder(&token, intents)
        .voice_manager_arc(manager)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
