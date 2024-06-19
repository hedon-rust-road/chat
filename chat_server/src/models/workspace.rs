use sqlx::PgPool;

use crate::{AppError, AppState};

use super::{ChatUser, Workspace};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
        INSERT INTO workspaces (name, owner_id)
        VALUES ($1, $2)
        RETURNING id, name, owner_id, created_at"#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }

    #[allow(unused)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1"#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1 order by id
        "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

impl Workspace {
    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"UPDATE workspaces
            SET owner_id = $1
            WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
            RETURNING id, name, owner_id, created_at"#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::CreateUser;

    use super::*;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await.unwrap();

        let input = CreateUser::new(&ws.name, "hedon", "hedon@example.com", "123456");
        let user = state.create_user(&input).await.unwrap();

        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &state.pool).await.unwrap();
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_id() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_id(1).await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let ws = state.find_workspace_by_name("acme").await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let users = state.fetch_chat_users(1).await?;
        assert_eq!(users.len(), 5);
        assert_eq!(users[0].email, "hedon@acme.com");
        assert_eq!(users[0].fullname, "Hedon");
        Ok(())
    }
}
