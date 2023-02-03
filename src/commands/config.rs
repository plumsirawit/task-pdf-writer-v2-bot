use std::{env, fs};

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
    let url_option = options
        .get(0)
        .expect("Expected url option")
        .resolved
        .as_ref()
        .expect("Expected url object");
    let reldir_option = options
        .get(1)
        .expect("Expected reldir option")
        .resolved
        .as_ref()
        .expect("Expected reldir object");
    let url = match url_option {
        CommandDataOptionValue::String(url) => url,
        _ => panic!("invalid url"),
    };
    let reldir = match reldir_option {
        CommandDataOptionValue::String(reldir) => reldir,
        _ => panic!("invalid reldir"),
    };
    let guild_id = command.guild_id.unwrap().to_string();
    sqlx::query!(
        "REPLACE INTO contests (guild_id, git_remote_url, contest_rel_path) VALUES (?, ?, ?)",
        guild_id,
        url,
        reldir
    )
    .execute(database) // < Where the command will be executed
    .await
    .unwrap();
    let repo_path = env::temp_dir().join(guild_id.to_string()).to_path_buf();
    fs::remove_dir_all(repo_path).unwrap();
    "OK, the URL is ".to_string() + url + " and the reldir is " + reldir
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
        .create_option(|option| {
            option
                .name("reldir")
                .description("Relative path to contest directory")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
