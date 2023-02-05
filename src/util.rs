use std::path::Path;
use std::{env, fs};

use git2::{AutotagOption, FetchOptions, MergeOptions, ObjectType, Repository};
use serenity::model::prelude::{Channel, ChannelId, GuildId};
use serenity::prelude::Context;
use uuid::Uuid;

use crate::traits::{MyError, TaskPdfWriterBotError};
use sqlx::FromRow;

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
        r#"SELECT COALESCE(guild_id, 'ID not found') AS "guild_id!: String", COALESCE(git_remote_url, 'URL not found') AS "git_remote_url!: String", COALESCE(contest_rel_path, 'relpath not found') AS "contest_rel_path!: String", private_key AS "private_key?: Vec<u8>" FROM contests WHERE guild_id = $1"#).bind(guild_id_string)
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
    println!("[DEBUG] {:#?}", repo_path.to_str());
    let exists = match repo_path.try_exists() {
        Ok(b) => b,
        Err(_) => false,
    };
    let pb = env::temp_dir().join(guild_id.to_string() + Uuid::new_v4().to_string().as_str());
    let repo: Repository = match exists {
        true => Repository::open(repo_path)?,
        false => {
            if let Some(k) = key {
                // https://github.com/rust-lang/git2-rs/issues/394

                //---------------------------------
                // build up auth credentials via fetch options:
                let mut cb = git2::RemoteCallbacks::new();

                let privkey_path: &Path = pb.as_path();
                if let Ok(()) = fs::write(&privkey_path, &k) {
                    println!(
                        "[DEBUG privkey] {}",
                        std::str::from_utf8(fs::read(&privkey_path)?.as_slice()).unwrap()
                    );
                    cb.credentials(|_, username_from_url, _cred| {
                        // trying https://github.com/rust-lang/git2-rs/issues/329
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
                    builder.clone(url.as_str(), &repo_path)?
                } else {
                    fs::remove_file(&privkey_path)?;
                    Err(MyError::new("cannot write to privkey_path"))?
                }
            } else {
                match Repository::clone(url.as_str(), repo_path) {
                    Ok(r) => r,
                    Err(e) => Err(MyError::new(
                        ("(probably your fault if the repo is private and you haven't set the private key) ".to_string()
                            + e.to_string().as_str())
                        .as_str()))?
                }
            }
        }
    };
    // need to make sure the borrow ends before returning
    {
        repo.remote_set_url("origin", url.as_str())?;
        let mut remote = repo
            .find_remote("origin")
            .or_else(|_| repo.remote_anonymous("origin"))?;

        // https://github.com/rust-lang/git2-rs/blob/master/examples/fetch.rs

        // Download the packfile and index it. This function updates the amount of
        // received data and the indexer stats which lets you inform the user about
        // progress.
        let mut fo = FetchOptions::new();
        remote.download(&[] as &[&str], Some(&mut fo))?;

        {
            // If there are local objects (we got a thin pack), then tell the user
            // how many objects we saved from having to cross the network.
            let stats = remote.stats();
            if stats.local_objects() > 0 {
                println!(
                    "\rReceived {}/{} objects in {} bytes (used {} local \
                 objects)",
                    stats.indexed_objects(),
                    stats.total_objects(),
                    stats.received_bytes(),
                    stats.local_objects()
                );
            } else {
                println!(
                    "\rReceived {}/{} objects in {} bytes",
                    stats.indexed_objects(),
                    stats.total_objects(),
                    stats.received_bytes()
                );
            }
        }

        // Disconnect the underlying connection to prevent from idling.
        remote.disconnect()?;

        // Update the references in the remote's namespace to point to the right
        // commits. This may be needed even if there was no packfile to download,
        // which can happen e.g. when the branches have been changed but all the
        // needed objects are available locally.
        remote.update_tips(None, true, AutotagOption::Unspecified, None)?;

        // in this block, we try to pull
        let our_commit = {
            let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
            match obj.into_commit() {
                Ok(c) => Ok(c),
                _ => Err(git2::Error::from_str("commit error")),
            }
        }?;
        let reference = repo.find_reference("FETCH_HEAD")?;
        let their_commit = reference.peel_to_commit()?;
        let _index = repo.merge_commits(&our_commit, &their_commit, Some(&MergeOptions::new()))?;
    }
    Ok(repo)
}
