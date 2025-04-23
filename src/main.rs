use std::collections::HashMap;
use std::env;

use clap::{Parser, Subcommand};
use reqwest::{header::AUTHORIZATION, StatusCode};
use serde::Deserialize;
use serenity::{
    all::{ChannelType, GuildId},
    builder::CreateChannel,
    prelude::*,
};

#[derive(Debug, Deserialize)]
struct Ratelimit {
    retry_after: f32,
}

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
    #[command(about = "Export Webhook")]
    Export { guild_id: u64 },
    #[command(about = "Create Webhook")]
    Webhook { guild_id: u64 },
}

#[derive(Debug, Deserialize)]
struct WebHook {
    url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged();

    let client = Client::builder(&token, intents)
        .await
        .expect("Error creating client");

    let webclient = reqwest::Client::builder()
        .user_agent("DiscordWebHookConfigurator/0.1.0")
        .build()
        .unwrap();

    match cli.command {
        Commands::Export { guild_id } => {
            let guild = GuildId::new(guild_id);
            let channels = guild.channels(&client.http).await.unwrap();

            let mut channels: Vec<_> = channels.iter().collect();

            channels.sort_by_cached_key(|(_id, chan)| {
                let (_, n) = chan.name.split_once('-').unwrap();
                n.parse::<u32>().unwrap()
            });

            for (cid, x) in channels {
                let url = loop {
                    let resp = webclient
                        .get(format!(
                            "https://discord.com/api/v10/channels/{cid}/webhooks"
                        ))
                        .header(AUTHORIZATION, format!("Bot {token}"))
                        .send()
                        .await
                        .unwrap();

                    let status = resp.status();

                    if status.is_success() {
                        let webhook: Vec<WebHook> = resp.json().await.unwrap();

                        break webhook.iter().next().unwrap().url.clone();
                    }

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let ratelimit: Ratelimit = resp.json().await.unwrap();
                        eprintln!("Ratelimit exceeded! sleep {}s", ratelimit.retry_after);
                        tokio::time::sleep(std::time::Duration::from_secs_f32(
                            ratelimit.retry_after,
                        ))
                        .await;
                        continue;
                    }

                    if status.is_client_error() {
                        panic!("UNKNOWN {status:?}");
                    } else {
                        continue;
                    }
                };

                println!("{} {}", x.name, url);
            }
        }
        Commands::Webhook { guild_id } => {
            let guild = GuildId::new(guild_id);
            let channels = guild.channels(&client.http).await.unwrap();

            let mut channels: Vec<_> = channels.iter().collect();

            channels.sort_by_cached_key(|(_id, chan)| {
                let (_, n) = chan.name.split_once('-').unwrap();
                n.parse::<u32>().unwrap()
            });

            for (cid, x) in channels {
                let already_exists = loop {
                    let resp = webclient
                        .get(format!(
                            "https://discord.com/api/v10/channels/{cid}/webhooks"
                        ))
                        .header(AUTHORIZATION, format!("Bot {token}"))
                        .send()
                        .await
                        .unwrap();

                    let status = resp.status();

                    if status.is_success() {
                        let webhook: Vec<WebHook> = resp.json().await.unwrap();

                        break webhook.iter().next().is_some();
                    }

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let ratelimit: Ratelimit = resp.json().await.unwrap();
                        eprintln!("Ratelimit exceeded! sleep {}s", ratelimit.retry_after);
                        tokio::time::sleep(std::time::Duration::from_secs_f32(
                            ratelimit.retry_after,
                        ))
                        .await;
                        continue;
                    }

                    if status.is_client_error() {
                        panic!("UNKNOWN {status:?}");
                    } else {
                        continue;
                    }
                };

                if already_exists {
                    eprintln!("Already Exists, for {} cancel.", x.name);
                    continue;
                }

                let url = loop {
                    let resp = webclient
                        .post(format!(
                            "https://discord.com/api/v10/channels/{cid}/webhooks"
                        ))
                        .header(AUTHORIZATION, format!("Bot {token}"))
                        .json(&{
                            let mut req = HashMap::new();
                            req.insert("name".to_string(), "explosion".to_string());
                            req
                        })
                        .send()
                        .await
                        .unwrap();

                    let status = resp.status();

                    if status.is_success() {
                        let webhook: WebHook = resp.json().await.unwrap();
                        break webhook.url;
                    }

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let ratelimit: Ratelimit = resp.json().await.unwrap();
                        eprintln!("Ratelimit exceeded! sleep {}s", ratelimit.retry_after);
                        tokio::time::sleep(std::time::Duration::from_secs_f32(
                            ratelimit.retry_after,
                        ))
                        .await;
                        continue;
                    }

                    if status.is_client_error() {
                        panic!("UNKNOWN {status:?}");
                    } else {
                        continue;
                    }
                };

                println!("{} {}", x.name, url);
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
