use std::{fmt, io};

use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::prelude::*;

pub struct CommandHandlerData<'a> {
    pub(super) command: &'a ApplicationCommandInteraction,
    pub(super) ctx: &'a Context,
    pub(super) database: &'a sqlx::SqlitePool,
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
        database: &'a sqlx::SqlitePool,
    ) -> CommandHandlerData<'a> {
        CommandHandlerData {
            command,
            ctx,
            database,
        }
    }
}

// struct CommandHandler<'a> {
//     data: CommandHandlerData<'a>,
// }
// #[async_trait]
// impl<'a> CommandHandle<'a> for CommandHandler<'a> {
//     async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError> {
//         Err(MyError::new("function 'handle' not implemented"))?
//     }
//     async fn run(&'a self) -> Result<String, TaskPdfWriterBotError> {
//         Err(MyError::new("function 'run' not implemented"))?
//     }
//     fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
//         command
//     }
// }

// #[macro_export]
// macro_rules! defer_handle {
//     ($self:ident) => {
//         async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError> {
//             $self
//                 .data
//                 .command
//                 .create_interaction_response(&$self.data.ctx.http, |response| {
//                     response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
//                 })
//                 .await?;
//             let file = $self.run().await?;
//             $self
//                 .data
//                 .command
//                 .create_followup_message(&$self.data.ctx.http, |response| {
//                     response.add_file(file.as_str())
//                 })
//                 .await?;
//             fs::remove_file(file.to_owned())?;

//             let content = $self.run().await?;
//             $self
//                 .data
//                 .command
//                 .create_interaction_response(&$self.data.ctx.http, |response| {
//                     response
//                         .kind(InteractionResponseType::ChannelMessageWithSource)
//                         .interaction_response_data(|message| message.content(content))
//                 })
//                 .await?;
//             Ok(())
//         }
//     };
// }

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
    Reqwest(reqwest::Error),
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
        TaskPdfWriterBotError::Reqwest(err)
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
            TaskPdfWriterBotError::Reqwest(s) => s.fmt(f),
        }
    }
}
