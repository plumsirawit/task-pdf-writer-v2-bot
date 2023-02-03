mod commands;

use std::env;

use futures::future::join_all;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::ChannelType;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command, &ctx).await,
                "genpdf" => commands::genpdf::run(&command.data.options).await,
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        for guild in ready.guilds.iter() {
            println!("{} is connected!", guild.id);
            let channel_id = get_channel_id(guild.id, &ctx).await;
            if channel_id.is_none() {
                println!("{} has no task-pdf-writer-bot channel, skipping.", guild.id);
                continue;
            }

            let commands = GuildId::set_application_commands(&guild.id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| commands::ping::register(command))
                    .create_application_command(|command| commands::genpdf::register(command))
            })
            .await;

            println!(
                "I now have the following guild slash commands on guild {}: {:#?}",
                guild.id, commands
            );
            // if !channel_names.contains(&"task-pdf-writer-bot".to_string()) {
            //     guild
            //         .id
            //         .create_channel(&ctx.http, |c| {
            //             c.name("task-pdf-writer-bot").kind(ChannelType::Text)
            //         })
            //         .await
            //         .unwrap();
            // }

            // let command_list = Command::get_global_application_commands(&ctx.http)
            //     .await
            //     .unwrap();

            // join_all(
            //     command_list.into_iter().map(|command| {
            //         Command::delete_global_application_command(&ctx.http, command.id)
            //     }),
            // )
            // .await
            // .into_iter()
            // .map(|x| x.unwrap())
            // .collect()
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

async fn get_channel_id(guild: GuildId, ctx: &Context) -> Option<ChannelId> {
    let channels = guild.channels(&ctx.http).await.unwrap();
    let bot_channel_id = channels
        .into_iter()
        .find(|(_k, v)| v.is_text_based() && v.name == "task-pdf-writer-bot".to_string());
    if bot_channel_id.is_none() {
        return None;
    }
    return Some(bot_channel_id.unwrap().0);
}
