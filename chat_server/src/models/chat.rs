use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{models::ChatType, AppError};

use super::Chat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

impl Chat {
    #[allow(unused)]
    pub async fn create(input: CreateChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let chat: Chat = sqlx::query_as(
            r#"INSERT INTO chats(ws_id, name, type, members)
            VALUES($1, $2, $3, $4)
            RETURNINGi id, ws_id, name, type, members, created_at"#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(ChatType::Group)
        .bind(input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
}
