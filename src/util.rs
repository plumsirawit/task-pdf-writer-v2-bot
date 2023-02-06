use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
use std::{env, fs};

use git2::{AutotagOption, FetchOptions, MergeOptions, ObjectType, Repository};
use serenity::model::prelude::{Channel, ChannelId, GuildId};
use serenity::prelude::Context;
use uuid::Uuid;

use crate::traits::{MyError, TaskPdfWriterBotError};
use sqlx::FromRow;

use tracing::{debug, info};

#[derive(FromRow)]
struct Contest {
    pub guild_id: String,
    pub git_remote_url: String,
    pub contest_rel_path: String,
    pub private_key: Option<Vec<u8>>,
}

pub async fn get_name(
    channel_id: ChannelId,
    ctx: &Context,
) -> Result<String, TaskPdfWriterBotError> {
    // Note: I don't know why, but calling
    // `command.channel_id.name(&ctx).await`
    // gives `None`.
    let channel_object = channel_id.to_channel(&ctx).await;
    match channel_object {
        Err(st) => Err(st)?,
        Ok(res) => match res {
            Channel::Guild(channel) => Ok(channel.name().to_string()),
            Channel::Category(category) => Ok(category.name().to_string()),
            Channel::Private(channel) => Ok(channel.name()),
            _ => Err(serenity::Error::Other("channel type not found"))?,
        },
    }
}

pub async fn get_metadata(
    guild_id: GuildId,
    database: &sqlx::PgPool,
) -> Result<(String, String, Option<Vec<u8>>), TaskPdfWriterBotError> {
    let guild_id_string = guild_id.to_string();
    let metadata: Result<Contest, sqlx::Error> = sqlx::query_as(
        r#"SELECT COALESCE(guild_id, 'ID not found') AS "guild_id", COALESCE(git_remote_url, 'URL not found') AS "git_remote_url", COALESCE(contest_rel_path, 'relpath not found') AS "contest_rel_path", private_key FROM contests WHERE guild_id = $1"#).bind(guild_id_string)
    .fetch_one(database)
    .await;
    return match metadata {
        Err(st) => Err(MyError::new(
            ("(probably your fault if you haven't config the bot) ".to_string()
                + st.to_string().as_str())
            .as_str(),
        ))?,
        Ok(res) => Ok((res.git_remote_url, res.contest_rel_path, res.private_key)),
    };
}

pub async fn prep_repo(
    guild_id: GuildId,
    url: String,
    key: Option<Vec<u8>>,
) -> Result<Repository, TaskPdfWriterBotError> {
    let repo_path = env::temp_dir().join(guild_id.to_string()).to_path_buf();
    debug!("repo_path {:#?}", repo_path.to_str());
    if repo_path.try_exists()? {
        fs::remove_dir_all(&repo_path)?;
    }
    let pb = env::temp_dir().join(guild_id.to_string() + Uuid::new_v4().to_string().as_str());
    let privkey_path: &Path = pb.as_path();
    if let Some(k) = key.clone() {
        if let Ok(()) = fs::write(&privkey_path, &k) {
            info!("Written privkey");
        } else {
            fs::remove_file(&privkey_path)?;
            Err(MyError::new("cannot write to privkey_path"))?
        }
        // https://github.com/rust-lang/git2-rs/issues/394

        //---------------------------------
        // build up auth credentials via fetch options:
        let mut cb = git2::RemoteCallbacks::new();
        cb.credentials(|_, username_from_url, _cred| {
            // trying https://github.com/rust-lang/git2-rs/issues/329
            debug!(
                "[DEBUG privkey] {}",
                std::str::from_utf8(fs::read(&privkey_path).unwrap().as_slice()).unwrap()
            );
            let user = username_from_url.unwrap_or("git");
            if _cred.contains(git2::CredentialType::USERNAME) {
                return git2::Cred::username(user);
            }
            let credentials = git2::Cred::ssh_key(user, None, privkey_path, None)?;
            Ok(credentials)
        });
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(cb);
        //---------------------------
        // Build builder
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);
        //-------------------
        // clone
        let repo = builder.clone(url.as_str(), &repo_path)?;
        fs::remove_file(&privkey_path)?;
        Ok(repo)
    } else {
        return match Repository::clone(url.as_str(), repo_path) {
                    Ok(r) => Ok(r),
                    Err(e) => Err(MyError::new(
                        ("(probably your fault if the repo is private and you haven't set the private key) ".to_string()
                            + e.to_string().as_str())
                        .as_str()))?
                };
    }
}
