use crate::lib::commands::{Event, Group};
use serde::{Deserialize, Serialize};
use tokio::prelude::*;

type Error = Box<dyn std::error::Error>;

const MDS_BASE_URI: &'static str = "https://mds.production.momentos.life";

#[derive(Deserialize, Serialize)]
pub struct LoginReq {
    email: String,
    password: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct LoginResp {
    pub jwt: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub privileges: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetUserGroupsResp {
    pub groups: Vec<Group>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetGroupedEventsResp {
    pub events: Vec<Event>,
}

pub struct MomentosClient {
    token: Option<String>,
    http_client: reqwest::Client,
}

impl MomentosClient {
    pub fn new() -> Self {
        MomentosClient {
            token: None,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_token(user_token: &str) -> Self {
        MomentosClient {
            token: Some(String::from(user_token)),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn login(&mut self, email: &str, password: &str) -> Result<LoginResp, Error> {
        let credentials = LoginReq {
            email: String::from(email),
            password: String::from(password),
        };

        let uri = format!("{}/login", MDS_BASE_URI);

        let resp = self
            .http_client
            .post(&uri)
            .json::<LoginReq>(&credentials)
            .send()
            .await?;

        let data = resp.json::<LoginResp>().await?;

        // update token for the client
        self.token.replace(data.jwt.clone());

        Ok(data)
    }

    pub async fn get_user_groups(&self, user_id: &str) -> Result<GetUserGroupsResp, Error> {
        let uri = format!("{}/api/v1/users/{}/groups", MDS_BASE_URI, user_id);

        let resp = self
            .http_client
            .get(&uri)
            .header(
                "Authorization",
                format!("Bearer {}", self.token.as_ref().unwrap()),
            )
            .send()
            .await?;

        let data = resp.json::<GetUserGroupsResp>().await?;

        Ok(data)
    }

    pub async fn get_grouped_events(&self, group_id: &str) -> Result<GetGroupedEventsResp, Error> {
        let uri = format!("{}/api/v1/groups/{}/events", MDS_BASE_URI, group_id);

        let resp = self
            .http_client
            .get(&uri)
            .header(
                "Authorization",
                format!("Bearer {}", self.token.as_ref().unwrap()),
            )
            .send()
            .await?;

        let data = resp.json::<GetGroupedEventsResp>().await?;

        Ok(data)
    }

    pub async fn get_event(&self, group_id: &str, event_id: &str) -> Result<Event, Error> {
        let uri = format!(
            "{}/api/v1/groups/{}/events/{}",
            MDS_BASE_URI, group_id, event_id
        );

        let queries = [
            ("fields", "title,recording,published,transcript"),
            ("presignedURL", "true"),
        ];

        let resp = self
            .http_client
            .get(&uri)
            .header(
                "Authorization",
                format!("Bearer {}", self.token.as_ref().unwrap()),
            )
            .query(&queries)
            .send()
            .await?;

        let data = resp.json::<Event>().await?;

        Ok(data)
    }

    pub async fn get_recording<W>(&self, uri: &str, writer: W) -> Result<usize, Error>
    where
        W: AsyncWrite,
        W: Unpin,
    {
        let mut writer = tokio::io::BufWriter::new(writer);
        let mut bytes: usize = 0;

        let mut resp = self.http_client.get(uri).send().await?;
        while let Some(chunk) = resp.chunk().await? {
            bytes += writer.write(&chunk).await?;
        }
        writer.flush().await?;

        Ok(bytes)
    }
}
