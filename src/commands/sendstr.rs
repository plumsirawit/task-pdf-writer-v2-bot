use crate::traits::{immediate_handle, CommandHandle, CommandHandlerData, TaskPdfWriterBotError};
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;

pub struct SendStrHandler<'a> {
    data: &'a CommandHandlerData<'a>,
    content: String,
}
impl<'a> SendStrHandler<'a> {
    pub fn new(data: &'a CommandHandlerData<'a>, content: String) -> SendStrHandler<'a> {
        SendStrHandler { data, content }
    }
}

#[async_trait]
impl<'a> CommandHandle<'a> for SendStrHandler<'a> {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command // Don't register this.
    }
    async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError> {
        let content = self.content.clone();
        immediate_handle(self.data, content).await
    }
}
