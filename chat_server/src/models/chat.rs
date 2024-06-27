use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{error::AppError, Chat, ChatType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

impl Chat {
    pub async fn create(input: CreateChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let chat = sqlx::query_as(
            "
            insert into chats (ws_id, name, type, members)
            values ($1, $2, $3, $4)
            returning id, name, members, created_at
            ",
        )
        .bind(ws_id as i64)
        .bind(&input.name)
        .bind(ChatType::Group)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }
}
