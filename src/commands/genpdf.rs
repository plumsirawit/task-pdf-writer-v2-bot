use crate::traits::{CommandHandle, CommandHandlerData, MyError, TaskPdfWriterBotError};
use crate::util::{get_metadata, get_name, prep_repo};

use base64::{engine::general_purpose, Engine};
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::InteractionResponseType;

use git2::Repository;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{env, fs};

fn retrieve_config(
    repo: &Repository,
    reldir: String,
) -> Result<serde_json::Value, TaskPdfWriterBotError> {
    let current_path = repo.path().join("..").join(reldir).join("config.json");
    println!("{}", current_path.display());
    let json_string = fs::read_to_string(current_path).expect("file read failed");
    return Ok(serde_json::from_str(json_string.as_str()).expect("JSON was not well-formatted"));
}

async fn generate_pdf(
    task_name: String,
    task_content: String,
    mut config_json: serde_json::Value,
) -> Result<PathBuf, TaskPdfWriterBotError> {
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
        Err(MyError::new("message in json is not a string"))?;
    }
    Ok(outfile_path)
}

pub struct GenpdfHandler<'a> {
    data: &'a CommandHandlerData<'a>,
}
impl<'a> GenpdfHandler<'a> {
    pub fn new(data: &'a CommandHandlerData<'a>) -> GenpdfHandler<'a> {
        GenpdfHandler { data }
    }
    async fn run(&'a self) -> Result<PathBuf, TaskPdfWriterBotError> {
        let name = get_name(self.data.command.channel_id, &self.data.ctx).await?;
        let (url, reldir) =
            get_metadata(self.data.command.guild_id.unwrap(), self.data.database).await?;
        let repo = prep_repo(self.data.command.guild_id.unwrap(), url).await?;
        let config_json = retrieve_config(&repo, reldir.to_owned()).expect("JSON retreival failed");
        let md_path = repo
            .path()
            .join("..")
            .join(reldir)
            .join(name.clone() + ".md");
        if !md_path.is_file() {
            Err(MyError::new("file not found"))?;
        }
        let file_content = String::from_utf8_lossy(&fs::read(md_path)?)
            .parse()
            .unwrap();
        generate_pdf(name.clone(), file_content, config_json.clone()).await
    }
}

#[async_trait]
impl<'a> CommandHandle<'a> for GenpdfHandler<'a> {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("genpdf")
            .description("Generates a PDF from markdown")
    }
    async fn handle(&'a self) -> Result<(), TaskPdfWriterBotError> {
        self.data
            .command
            .create_interaction_response(&self.data.ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await?;
        match self.run().await {
            Ok(file) => {
                self.data
                    .command
                    .create_followup_message(&self.data.ctx.http, |response| {
                        response.add_file(&file)
                    })
                    .await?;
                fs::remove_file(file.to_owned())?;
            }
            Err(e) => {
                self.data
                    .command
                    .create_followup_message(&self.data.ctx.http, |response| {
                        response.content(e.to_string().as_str())
                    })
                    .await?;
            }
        };
        Ok(())
    }
}

// The following stupid comment is for use in case of generating multiple files at the same time.

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
