use clap::{Parser, Subcommand};
use serenity::{
    all::{ChannelType, GuildId},
    builder::{CreateChannel, CreateWebhook},
    prelude::*,
};
use std::env;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Create explosion")]
    Explosion { guild_id: u64 },
    #[command(about = "Create Webhook")]
    Webhook { guild_id: u64 },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged();

    let client = Client::builder(token, intents)
        .await
        .expect("Error creating client");

    match cli.command {
        Commands::Webhook { guild_id } => {
            let guild = GuildId::new(guild_id);
            let channels = guild.channels(&client.http).await.unwrap();

            for (cid, x) in channels {
                let url = loop {
                    let w = CreateWebhook::new("Explosion");
                    match cid.create_webhook(&client.http, w).await {
                        Ok(webhook) => break webhook.url(),
                        Err(e) => {
                            eprintln!("Create channel error {e:?}... wait 15s...");
                        }
                    }
                };

                println!("{} {}", x.name, url.unwrap());
            }
        }
        Commands::Explosion { guild_id } => {
            let guild = GuildId::new(guild_id);
            for n in 1..=500 {
                let channel = loop {
                    let c = CreateChannel::new(format!("channel-{n}")).kind(ChannelType::Text);

                    match guild.create_channel(&client.http, c).await {
                        Ok(category) => break category.id,
                        Err(e) => {
                            eprintln!("Create channel error {e:?}... wait 5s...");
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                };

                eprintln!("Channel #{n} Created!");
                println!("{channel}");
            }
        }
    }
}
