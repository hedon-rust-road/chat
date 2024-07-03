use std::{net::SocketAddr, time::Duration, vec};

use chat_core::{Chat, ChatType, Message};
use futures::StreamExt;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::json;
use tokio::{net::TcpListener, time::sleep};

const WILD_ADDR: &str = "0.0.0.0:0";

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

struct NotifyServer;

#[tokio::test]
async fn chat_server_should_work() -> anyhow::Result<()> {
    let (tdb, state) = chat_server::AppState::new_for_test().await?;
    let cs = ChatServer::new(state).await?;
    let db_url = tdb.url();
    NotifyServer::new(&db_url, &cs.token).await?;
    let chat = cs.create_chat().await?;
    let _msg = cs.create_message(chat.id as u64).await?;
    sleep(Duration::from_secs(1)).await;
    Ok(())
}

impl ChatServer {
    async fn new(state: chat_server::AppState) -> anyhow::Result<Self> {
        let app = chat_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();
        let mut ret = Self {
            addr,
            client,
            token: "".to_string(),
        };
        ret.token = ret.signin().await?;

        Ok(ret)
    }

    async fn signin(&self) -> anyhow::Result<String> {
        let res = self
            .client
            .post(&format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(r#"{"email": "hedon@acme.com", "password": "123456"}"#)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);
        let ret: AuthToken = res.json().await?;

        Ok(ret.token)
    }

    async fn create_chat(&self) -> anyhow::Result<Chat> {
        let res = self
            .client
            .post(&format!("http://{}/api/chats", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(r#"{"name": "test", "members": [1,2], "public": false}"#)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);
        let chat: Chat = res.json().await?;
        assert_eq!(chat.name.as_ref().unwrap(), "test");
        assert_eq!(chat.members, vec![1, 2]);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        Ok(chat)
    }

    async fn create_message(&self, chat_id: u64) -> anyhow::Result<Message> {
        // upload file
        let data = include_bytes!("../Cargo.toml");
        let files = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);

        let res = self
            .client
            .post(&format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let ret: Vec<String> = res.json().await?;
        let body = serde_json::to_string(&json!({
            "content": "hello",
            "files": ret,
        }))?;
        let res = self
            .client
            .post(format!("http://{}/api/chats/{}", self.addr, chat_id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let msg: Message = res.json().await?;
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.files, ret);
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.chat_id, chat_id as i64);
        Ok(msg)
    }
}

impl NotifyServer {
    async fn new(db_url: &str, token: &str) -> anyhow::Result<Self> {
        let mut config = notify_server::AppConfig::load()?;
        config.server.db_url = db_url.to_string();
        let app = notify_server::get_router(config).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));

        tokio::spawn(async move {
            // NOTE: next() need `futures` crate
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),
                    Ok(Event::Message(message)) => match message.event.as_str() {
                        "NewChat" => {
                            let chat: Chat = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(chat.name.as_ref().unwrap(), "test");
                            assert_eq!(chat.members, vec![1, 2]);
                            assert_eq!(chat.r#type, ChatType::PrivateChannel);
                        }
                        "NewMessage" => {
                            let msg: Message = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(msg.content, "hello");
                            assert_eq!(msg.files.len(), 1);
                            assert_eq!(msg.sender_id, 1);
                        }
                        _ => {
                            panic!("unexpected event: {:?}", message);
                        }
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                        es.close();
                    }
                }
            }
        });
        Ok(Self)
    }
}
