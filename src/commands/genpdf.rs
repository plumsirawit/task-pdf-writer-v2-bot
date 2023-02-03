use crate::util::{get_name, get_url, prep_repo};

use base64::{engine::general_purpose, Engine};
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

use git2::Repository;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{env, fs};

fn retrieve_config(repo: &Repository) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let json_string =
        fs::read_to_string(repo.path().join("..").join("contest").join("config.json"))
            .expect("file read failed");
    return Ok(serde_json::from_str(json_string.as_str()).expect("JSON was not well-formatted"));
}

async fn generate_pdf(
    task_name: String,
    task_content: String,
    mut config_json: serde_json::Value,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let mut hasher = DefaultHasher::new();
    (task_name.clone() + task_content.as_str()).hash(&mut hasher);
    let hashed_out = hasher.finish();
    let outfile_path = env::temp_dir().join(hashed_out.to_string() + ".pdf");
    config_json["content"] = serde_json::Value::String(task_content);
    config_json["task_name"] = serde_json::Value::String(task_name);
    let client = reqwest::Client::new();
    let resp = client
        .post("https://973i5k6wjg.execute-api.ap-southeast-1.amazonaws.com/dev/genpdf")
        .body(config_json.to_string())
        .send()
        .await?
        .text()
        .await?;
    let resp_obj: serde_json::Value =
        serde_json::from_str(resp.as_str()).expect("JSON parse from fetch is fine");
    let resp_obj = resp_obj.as_object().expect("the resp is an object");
    if let serde_json::Value::String(content_base64) =
        resp_obj.get("message").expect("message exists")
    {
        let decoded_content = general_purpose::STANDARD.decode(content_base64).unwrap();
        fs::write(&outfile_path, decoded_content).unwrap();
    } else {
        return Err("message in json is not a string".into());
    }
    return Ok(outfile_path.to_str().unwrap().to_string());
}
pub async fn run(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    database: &sqlx::SqlitePool,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let name = get_name(command.channel_id, &ctx)
        .await
        .expect("channel name");
    let url = get_url(command.guild_id.unwrap(), database)
        .await
        .expect("get url");
    let repo = prep_repo(command.guild_id.unwrap(), url)
        .await
        .expect("repo prepped");
    let config_json = retrieve_config(&repo).expect("JSON retreival failed");
    let md_path = repo
        .path()
        .join("..")
        .join("contest")
        .join(name.clone() + ".md");
    if !md_path.is_file() {
        return Err("file not found".into());
    }
    let file_content = String::from_utf8_lossy(&fs::read(md_path)?)
        .parse()
        .unwrap();
    generate_pdf(name.clone(), file_content, config_json.clone()).await
    // for entry in repo
    //     .path()
    //     .join("..")
    //     .join("contest")
    //     .read_dir()
    //     .expect("directory is read")
    // {
    //     if let Ok(ent) = entry {
    //         let file_name = ent.file_name().to_str().expect("to str").to_string();
    //         if !file_name.ends_with(".md") {
    //             continue;
    //         }
    //         let file_name = file_name
    //             .strip_suffix(".md")
    //             .expect(".md can be stripped")
    //             .to_string();
    //         let file_content = String::from_utf8_lossy(&fs::read(ent.path())?)
    //             .parse()
    //             .expect("file read success");
    //         file_futures.push(generate_pdf(file_name, file_content, config_json.clone()));
    //     }
    // }
    // let pdf_result: Vec<String> = join_all(file_futures)
    //     .await
    //     .iter()
    //     .map(|x| x.as_ref().expect("OK").clone())
    //     .collect();

    // return Ok(pdf_result);
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("genpdf")
        .description("Generates a PDF from markdown")
}
