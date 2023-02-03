use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub async fn run(_options: &[CommandDataOption]) -> String {
    "Hello, world!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("genpdf")
        .description("Generates a PDF from markdown")
}
