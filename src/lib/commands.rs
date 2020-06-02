use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io,
    prelude::*,
};

extern crate clap;
use crate::lib::api::{GetGroupedEventsResp, GetUserGroupsResp, LoginResp, MomentosClient};
use crate::lib::settings::Settings;

type Error = Box<dyn std::error::Error>;

pub async fn login(settings: &mut Settings, email: &str) -> Result<(), Error> {
    let password =
        rpassword::read_password_from_tty(Some("Password: ")).expect("Failed to read password");

    let mut client = MomentosClient::new();
    let user_access: LoginResp = client.login(email, password.as_str()).await?;

    // update settings and save them to file
    settings.set_token(&user_access.jwt);
    settings.set_uid(&user_access.id);
    settings.save_to_file()?;

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Group {
    #[serde(rename = "_id")]
    id: String,
    name: String,
}

pub async fn list(settings: &Settings) -> Result<(), Error> {
    let token = settings.get_token().expect("User access token not found");
    let uid = settings.get_uid().expect("User ID not found");
    let client = MomentosClient::with_token(token);
    let user_groups: GetUserGroupsResp = client.get_user_groups(uid).await?;

    for g in &user_groups.groups {
        println!("{} '{}'", g.id, g.name);
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
struct Recording {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "presignedURL")]
    presigned_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Phrase {
    #[serde(rename = "_id")]
    id: String,
    text: String,
    #[serde(rename = "timeInterval")]
    time_interval: (f32, f32),
}

fn format_seconds(_seconds: f32) -> String {
    let mut input = _seconds as u32;

    let hours = input / 3600;
    input -= hours * 3600;

    let minutes = input / 60;
    input -= minutes * 60;

    let seconds = input;

    let milliseconds = (_seconds.fract() * 1000.0) as u32;

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours, minutes, seconds, milliseconds
    )
}

impl Phrase {
    fn as_vtt_string(&self) -> String {
        let mut s = String::new();

        s.push_str(&format!(
            "{} --> {}\n",
            format_seconds(self.time_interval.0),
            format_seconds(self.time_interval.1)
        ));
        s.push_str(&self.text);
        s.push_str("\n\n");

        s
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Transcript {
    phrases: Vec<Phrase>,
}

impl Transcript {
    async fn write_as_vtt<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: AsyncWrite,
        W: Unpin,
    {
        let mut bytes = 0;
        let mut writer = io::BufWriter::new(writer);
        writer.write(b"WEBVTT\n\n").await?;
        for phrase in self.phrases.iter() {
            bytes += writer.write(phrase.as_vtt_string().as_bytes()).await?;
        }
        writer.flush().await?;
        Ok(bytes)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    #[serde(rename = "_id")]
    id: String,
    title: String,
    recording: Recording,
    published: bool,
    transcript: Option<Transcript>,
}

pub async fn download(settings: &Settings, groups: &Vec<&str>) -> Result<(), Error> {
    let token = settings.get_token().expect("User access token not found");
    let client = MomentosClient::with_token(token);

    for gid in groups {
        println!("processing group {} ...", gid);

        // At this point, the event is missing the transcript and recording url
        let partial_events: GetGroupedEventsResp = client.get_grouped_events(gid).await?;

        // Create folder to place group's contents
        create_dir_all(gid).await.expect("Failed to create folder");

        // Creating a index file that maps {event.id -> event.title} since titles may not be valid filename
        let dir_path = std::path::Path::new(gid);
        let file_path = std::path::Path::new("index.csv");
        let path = std::path::Path::join(dir_path, file_path);
        write_index_to_file(&partial_events.events, path.to_str().unwrap()).await?;

        let futures: Vec<_> = partial_events
            .events
            .iter()
            .map(|e| download_event_contents(&client, &gid, &e.id))
            .collect();

        join_all(futures).await;
    }

    Ok(())
}

enum FileParams {
    FileName(String),
    FileWithExt { name: String, ext: String },
}

async fn create_file_in_folder(folder_name: &str, file_params: &FileParams) -> Result<File, Error> {
    let dir_path = std::path::Path::new(folder_name);
    let file_path = match file_params {
        FileParams::FileName(filename) => filename.into(),
        FileParams::FileWithExt { name, ext } => format!("{}.{}", name, ext),
    };
    let path = std::path::Path::join(dir_path, file_path);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)
        .await?;

    Ok(file)
}

async fn write_index_to_file(events: &Vec<Event>, file_path: &str) -> Result<(), Error> {
    let f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .await?;

    let mut writer = io::BufWriter::new(f);

    writer.write(b"ID,Title,Published\n").await?;
    for event in events.iter() {
        let buffer = format!("{},{},{}\n", event.id, event.title, event.published);
        writer.write(buffer.as_bytes()).await?;
    }
    writer.flush().await?;

    Ok(())
}

async fn download_event_contents(
    client: &MomentosClient,
    group_id: &str,
    event_id: &str,
) -> Result<(), Error> {
    println!("downloading event {}", event_id);

    let event: Event = client.get_event(group_id, event_id).await?;

    // Writing transcript to a file
    if let Some(transcript) = &event.transcript {
        let file_params = FileParams::FileWithExt {
            name: event_id.into(),
            ext: "vtt".into(),
        };
        let file = create_file_in_folder(group_id, &file_params).await?;
        transcript.write_as_vtt(file).await?;
    }

    // Writing recoding to a file
    if let Some(recording_url) = &event.recording.presigned_url {
        let file_params = FileParams::FileWithExt {
            name: event_id.into(),
            ext: "mp4".into(),
        };
        let file = create_file_in_folder(group_id, &file_params).await?;
        client.get_recording(recording_url, file).await?;
    }

    println!("done with {}", event_id);

    Ok(())
}
