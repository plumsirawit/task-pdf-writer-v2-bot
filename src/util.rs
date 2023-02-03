use std::env;

use git2::{MergeOptions, ObjectType, Repository};
use serenity::model::prelude::{Channel, ChannelId, GuildId};
use serenity::prelude::Context;

pub async fn get_name(channel_id: ChannelId, ctx: &Context) -> Result<String, serenity::Error> {
    // Note: I don't know why, but calling
    // `command.channel_id.name(&ctx).await`
    // gives `None`.
    let channel_object = channel_id.to_channel(&ctx).await;
    match channel_object {
        Err(st) => Err(st),
        Ok(res) => match res {
            Channel::Guild(channel) => Ok(channel.name().to_string()),
            Channel::Category(category) => Ok(category.name().to_string()),
            Channel::Private(channel) => Ok(channel.name()),
            _ => Err(serenity::Error::Other("Not found")),
        },
    }
}

pub async fn get_url(
    guild_id: GuildId,
    database: &sqlx::SqlitePool,
) -> Result<String, sqlx::Error> {
    let guild_id_string = guild_id.to_string();
    let git_remote_url = sqlx::query!(
        "SELECT git_remote_url FROM contests WHERE guild_id = ?",
        guild_id_string
    )
    .fetch_one(database) // < Where the command will be executed
    .await;
    return match git_remote_url {
        Err(st) => Err(st),
        Ok(res) => Ok(res.git_remote_url),
    };
}

pub async fn prep_repo(guild_id: GuildId, url: String) -> Result<Repository, git2::Error> {
    let repo_path = env::temp_dir().join(guild_id.to_string()).to_path_buf();
    println!("[DEBUG] {:#?}", repo_path.to_str());
    let exists = match repo_path.try_exists() {
        Ok(b) => b,
        Err(_) => false,
    };
    let repo = match exists {
        true => Repository::open(repo_path),
        false => Repository::clone(url.as_str(), repo_path),
    };
    if let Ok(rep) = &repo {
        // in this block, we try to pull
        let our_commit = {
            let obj = rep
                .head()
                .unwrap()
                .resolve()
                .unwrap()
                .peel(ObjectType::Commit)?;
            match obj.into_commit() {
                Ok(c) => Ok(c),
                _ => Err(git2::Error::from_str("commit error")),
            }
        }
        .unwrap();
        let reference = rep.find_reference("HEAD").unwrap();
        let their_commit = reference.peel_to_commit().unwrap();
        let _index = rep.merge_commits(&our_commit, &their_commit, Some(&MergeOptions::new()))?;
    }
    return repo;
}
