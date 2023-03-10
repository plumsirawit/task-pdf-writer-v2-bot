use std::{fmt, io};

use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::prelude::*;

pub struct CommandHandlerData<'a> {
    pub(super) command: &'a ApplicationCommandInteraction,
    pub(super) ctx: &'a Context,
    pub(super) database: &'a sqlx::PgPool,
}

#[async_trait]
pub trait CommandHandle<'a> {
    async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError>;
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

impl<'a> CommandHandlerData<'a> {
    pub fn new(
        command: &'a ApplicationCommandInteraction,
        ctx: &'a Context,
        database: &'a sqlx::PgPool,
    ) -> CommandHandlerData<'a> {
        CommandHandlerData {
            command,
            ctx,
            database,
        }
    }
}

pub async fn immediate_handle<'a>(
    data: &CommandHandlerData<'a>,
    content: String,
) -> Result<(), TaskPdfWriterBotError> {
    data.command
        .create_interaction_response(&data.ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await?;
    Ok(())
}

#[derive(Debug)]
pub struct MyError {
    why: String,
}
impl MyError {
    pub fn new(why: &str) -> Self {
        MyError {
            why: why.to_string(),
        }
    }
}

impl std::error::Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[BOT INTERNAL ERROR] {}", self.why)
    }
}

#[derive(Debug)]
pub enum TaskPdfWriterBotError {
    SerenityError(serenity::Error),
    InnerError(MyError),
    SqlxError(sqlx::Error),
    GitError(git2::Error),
    IOError(io::Error),
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    OpenSSHError(openssh::Error),
}
impl std::error::Error for TaskPdfWriterBotError {}
impl From<MyError> for TaskPdfWriterBotError {
    fn from(err: MyError) -> Self {
        TaskPdfWriterBotError::InnerError(err)
    }
}
impl From<serenity::Error> for TaskPdfWriterBotError {
    fn from(err: serenity::Error) -> Self {
        TaskPdfWriterBotError::SerenityError(err)
    }
}
impl From<sqlx::Error> for TaskPdfWriterBotError {
    fn from(err: sqlx::Error) -> Self {
        TaskPdfWriterBotError::SqlxError(err)
    }
}
impl From<git2::Error> for TaskPdfWriterBotError {
    fn from(err: git2::Error) -> Self {
        TaskPdfWriterBotError::GitError(err)
    }
}
impl From<io::Error> for TaskPdfWriterBotError {
    fn from(err: io::Error) -> Self {
        TaskPdfWriterBotError::IOError(err)
    }
}
impl From<reqwest::Error> for TaskPdfWriterBotError {
    fn from(err: reqwest::Error) -> Self {
        TaskPdfWriterBotError::ReqwestError(err)
    }
}
impl From<serde_json::Error> for TaskPdfWriterBotError {
    fn from(err: serde_json::Error) -> Self {
        TaskPdfWriterBotError::JsonError(err)
    }
}
impl From<openssh::Error> for TaskPdfWriterBotError {
    fn from(err: openssh::Error) -> Self {
        TaskPdfWriterBotError::OpenSSHError(err)
    }
}
impl fmt::Display for TaskPdfWriterBotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskPdfWriterBotError::SerenityError(s) => s.fmt(f),
            TaskPdfWriterBotError::InnerError(s) => s.fmt(f),
            TaskPdfWriterBotError::SqlxError(s) => s.fmt(f),
            TaskPdfWriterBotError::GitError(s) => s.fmt(f),
            TaskPdfWriterBotError::IOError(s) => s.fmt(f),
            TaskPdfWriterBotError::ReqwestError(s) => s.fmt(f),
            TaskPdfWriterBotError::JsonError(s) => s.fmt(f),
            TaskPdfWriterBotError::OpenSSHError(s) => s.fmt(f)
        }
    }
}
