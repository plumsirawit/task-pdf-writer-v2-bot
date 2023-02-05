use std::{env, fs};

use crate::traits::{CommandHandle, CommandHandlerData, MyError, TaskPdfWriterBotError};

use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::interaction::InteractionResponseType;

pub struct ConfigHandler<'a> {
    data: &'a CommandHandlerData<'a>,
}
impl<'a> ConfigHandler<'a> {
    pub fn new(data: &'a CommandHandlerData<'a>) -> ConfigHandler<'a> {
        ConfigHandler { data }
    }
    async fn run(&self) -> Result<String, TaskPdfWriterBotError> {
        let options = &self.data.command.data.options;
        let url_option = match options[0].resolved.as_ref() {
            Some(s) => s,
            None => Err(MyError::new("(probably your fault): options[0] not found"))?,
        };
        let reldir_option = match options[1].resolved.as_ref() {
            Some(s) => s,
            None => Err(MyError::new("(probably your fault): options[1] not found"))?,
        };
        let url = match url_option {
            CommandDataOptionValue::String(url) => url,
            _ => Err(MyError::new("(probably your fault): invalid url"))?,
        };
        let reldir = match reldir_option {
            CommandDataOptionValue::String(reldir) => reldir,
            _ => Err(MyError::new("(probably your fault): invalid reldir"))?,
        };
        let privkey = options[2].resolved.as_ref();
        let guild_id = match self.data.command.guild_id {
            Some(s) => s,
            None => Err(MyError::new("guild_id not found"))?,
        }
        .to_string();
        match privkey {
            Some(privkey_value) => {
                let pk = match privkey_value {
                    CommandDataOptionValue::Attachment(pk) => pk,
                    _ => Err(MyError::new(
                        "(probably your fault): private key is not an attachment",
                    ))?,
                };
                let downloaded_attachment = pk.download().await?;
                sqlx::query(
                    "INSERT INTO contests (guild_id, git_remote_url, contest_rel_path, private_key) VALUES ($1, $2, $3, $4) ON CONFLICT (guild_id) DO UPDATE SET git_remote_url = EXCLUDED.git_remote_url, contest_rel_path = EXCLUDED.contest_rel_path, private_key = EXCLUDED.private_key")
                .bind(&guild_id)
                .bind(&url)
                .bind(&reldir)
                .bind(&downloaded_attachment)
                .execute(self.data.database) // < Where the command will be executed
                .await?;
            }
            None => {
                sqlx::query(
                    "INSERT INTO contests (guild_id, git_remote_url, contest_rel_path) VALUES ($1, $2, $3) ON CONFLICT (guild_id) DO UPDATE SET git_remote_url = EXCLUDED.git_remote_url, contest_rel_path = EXCLUDED.contest_rel_path")
                    .bind(&guild_id)
                    .bind(&url)
                    .bind(&reldir)
                .execute(self.data.database) // < Where the command will be executed
                .await?;
            }
        }
        let repo_path = env::temp_dir().join(guild_id.to_string()).to_path_buf();
        if repo_path.is_dir() {
            fs::remove_dir_all(repo_path)?;
        }
        Ok("OK, the URL is ".to_string() + url + " and the reldir is " + reldir)
    }
}

#[async_trait]
impl<'a> CommandHandle<'a> for ConfigHandler<'a> {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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
            .create_option(|option| {
                option
                    .name("privkey")
                    .description("Private key for git repository")
                    .kind(CommandOptionType::Attachment)
                    .required(false)
            })
    }
    async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError> {
        self.data
            .command
            .create_interaction_response(&self.data.ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await?;
        let retst = match self.run().await {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        self.data
            .command
            .create_followup_message(&self.data.ctx.http, |response| response.content(retst))
            .await?;
        Ok(())
    }
}
