use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::prelude::Context;

pub async fn run(
    command: &ApplicationCommandInteraction,
    _ctx: &Context,
    database: &sqlx::SqlitePool,
) -> String {
    let options = &command.data.options;
    let option = options
        .get(0)
        .expect("Expected url option")
        .resolved
        .as_ref()
        .expect("Expected url object");
    if let CommandDataOptionValue::String(url) = option {
        let guild_id = command.guild_id.unwrap().to_string();
        sqlx::query!(
            "REPLACE INTO contests (guild_id, git_remote_url) VALUES (?, ?)",
            guild_id,
            url
        )
        .execute(database) // < Where the command will be executed
        .await
        .unwrap();
        "OK, the URL is ".to_string() + url
    // Do some sanitary check on url (maybe try actual cloning)
    } else {
        "Please provide a valid url".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("config")
        .description("Configures the git repository to task-pdf-writer-v2-bot")
        .create_option(|option| {
            option
                .name("url")
                .description("Git URL")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
