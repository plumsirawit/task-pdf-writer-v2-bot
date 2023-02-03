use crate::util::{get_metadata, get_name};

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{Channel, ChannelType};
use serenity::prelude::Context;

pub async fn run(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    database: &sqlx::SqlitePool,
) -> String {
    let channel_object = command.channel_id.to_channel(&ctx).await.unwrap();
    let kind = match channel_object {
        Channel::Guild(channel) => channel.kind,
        Channel::Category(channel) => channel.kind,
        Channel::Private(channel) => channel.kind,
        _ => ChannelType::Unknown,
    };
    if kind == ChannelType::PublicThread || kind == ChannelType::PrivateThread {
        command.channel_id.join_thread(&ctx.http).await.unwrap();
    }
    let name = get_name(command.channel_id, &ctx).await;
    let mdata = get_metadata(command.guild_id.unwrap(), database).await;
    return {
        (match name {
            Err(e) => e.to_string(),
            Ok(s) => s,
        }) + " | "
            + (match mdata {
                Err(e) => e.to_string(),
                Ok((s, t)) => s + ", " + t.as_str(),
            })
            .as_str()
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ping")
        .description("A ping command, for debug-related purposes only.")
}
