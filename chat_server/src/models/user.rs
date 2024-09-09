use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chat_core::{ChatUser, User};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppError, AppState};

/// create a user with email and password
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct CreateUser {
    /// Email of the user
    pub email: String,
    /// Full name of the user
    pub fullname: String,
    /// Workspace name - this will be used to create a workspace if it doesn't exist
    pub workspace: String,
    /// Password of the user
    pub password: String,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    // TODO: use transaction for workspace creation and user creation.
    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        // check if email exists
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        };

        // check if workspace exists, if not, create one
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        // create a new user
        let password_hash = hash_password(&input.password)?;
        let mut user: User = sqlx::query_as(
            r#"
            INSERT INTO users(ws_id, email, fullname, password_hash)
            VALUES($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(ws.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        user.ws_name = ws.name.clone();

        // update workspace owner
        if ws.owner_id == 0 {
            self.update_workspace_owner(ws.id as _, user.id as _)
                .await?;
        }

        Ok(user)
    }

    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    // load ws_name, ws should exist
                    let ws = self.find_workspace_by_id(user.ws_id as _).await?.unwrap();
                    user.ws_name = ws.name;
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    #[allow(unused)]
    pub async fn fetch_chat_user_by_id(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)"#,
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    #[allow(dead_code)]
    pub async fn fetch_chat_users(&self, ws_id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1"#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let slat = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &slat)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
impl CreateUser {
    pub fn new(ws: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            workspace: ws.to_string(),
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::verify_password;

    #[test]
    fn hash_password_and_verify_should_work() -> anyhow::Result<()> {
        let password = "hedon";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "hedon@gmail.com";
        let fullname = "Hedon";
        let password = "hedon";
        let input = CreateUser::new("none", fullname, email, password);
        let user = state.create_user(&input).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let signin_user = SigninUser::new(email, password);
        let user = state.verify_user(&signin_user).await?;
        assert!(user.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn create_duplicated_user_should_fail() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("none", "Hedon", "hedon@gmail.com", "hedon");

        state.create_user(&input).await?;
        let ret = state.create_user(&input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, "hedon@gmail.com");
            }
            _ => panic!("should fail"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_id_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        Ok(())
    }
}
