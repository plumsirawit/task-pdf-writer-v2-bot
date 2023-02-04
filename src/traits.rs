use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

pub struct CommandHandler<'a> {
    command: &'a ApplicationCommandInteraction,
    ctx: &'a Context,
    database: &'a sqlx::SqlitePool,
}
#[async_trait]
pub trait CommandHandle {
    async fn handle(&self) {}
    async fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand { command }
}
