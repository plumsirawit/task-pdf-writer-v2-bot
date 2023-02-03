use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{Channel, ChannelId};
use serenity::prelude::Context;

async fn get_name(channel_id: ChannelId, ctx: &Context) -> String {
    // Note: I don't know why, but calling
    // `command.channel_id.name(&ctx).await`
    // gives `None`.
    let channel_object = channel_id.to_channel(&ctx).await;
    match channel_object {
        Err(st) => st.to_string(),
        Ok(res) => match res {
            Channel::Guild(channel) => channel.name().to_string(),
            Channel::Category(category) => category.name().to_string(),
            Channel::Private(channel) => channel.name(),
            _ => "Not found".to_string(),
        },
    }
}

pub async fn run(command: &ApplicationCommandInteraction, ctx: &Context) -> String {
    command.channel_id.join_thread(&ctx.http).await.unwrap();
    let name = get_name(command.channel_id, &ctx).await;
    // query pdf status for `name`
    return name;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}
