use std::env;

use git2::{AutotagOption, FetchOptions, MergeOptions, ObjectType, Repository};
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

pub async fn get_metadata(
    guild_id: GuildId,
    database: &sqlx::SqlitePool,
) -> Result<(String, String), sqlx::Error> {
    let guild_id_string = guild_id.to_string();
    let metadata = sqlx::query!(
        "SELECT git_remote_url, contest_rel_path FROM contests WHERE guild_id = ?",
        guild_id_string
    )
    .fetch_one(database) // < Where the command will be executed
    .await;
    return match metadata {
        Err(st) => Err(st),
        Ok(res) => Ok((res.git_remote_url, res.contest_rel_path)),
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
        rep.remote_set_url("origin", url.as_str()).unwrap();
        let mut remote = rep
            .find_remote("origin")
            .or_else(|_| rep.remote_anonymous("origin"))?;

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
        let reference = rep.find_reference("FETCH_HEAD").unwrap();
        let their_commit = reference.peel_to_commit().unwrap();
        let _index = rep.merge_commits(&our_commit, &their_commit, Some(&MergeOptions::new()))?;
    }
    return repo;
}
