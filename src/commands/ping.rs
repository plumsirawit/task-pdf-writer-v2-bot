use crate::traits::{immediate_handle, CommandHandle, CommandHandlerData, TaskPdfWriterBotError};
use crate::util::{get_metadata, get_name};

use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::{Channel, ChannelType};

pub struct PingHandler<'a> {
    data: &'a CommandHandlerData<'a>,
}
impl<'a> PingHandler<'a> {
    pub fn new(data: &'a CommandHandlerData<'a>) -> PingHandler<'a> {
        PingHandler { data }
    }
    async fn run(&'a self) -> Result<String, TaskPdfWriterBotError> {
        let channel_object = self
            .data
            .command
            .channel_id
            .to_channel(&self.data.ctx)
            .await?;
        let kind = match channel_object {
            Channel::Guild(channel) => channel.kind,
            Channel::Category(channel) => channel.kind,
            Channel::Private(channel) => channel.kind,
            _ => ChannelType::Unknown,
        };
        if kind == ChannelType::PublicThread || kind == ChannelType::PrivateThread {
            self.data
                .command
                .channel_id
                .join_thread(&self.data.ctx.http)
                .await?;
        }
        let name = get_name(self.data.command.channel_id, &self.data.ctx).await;
        let mdata = get_metadata(self.data.command.guild_id.unwrap(), self.data.database).await?;
        Ok(name? + " | " + mdata.0.as_str() + " | " + mdata.1.as_str())
    }
}

#[async_trait]
impl<'a> CommandHandle<'a> for PingHandler<'a> {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("ping")
            .description("A ping command, for debug-related purposes only.")
    }
    async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError> {
        let content = match self.run().await {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        immediate_handle(self.data, content).await
    }
}
