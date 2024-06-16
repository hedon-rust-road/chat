mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use anyhow::Context;
pub use config::AppConfig;
pub use error::AppError;
use handlers::*;
use middlewares::{set_layer, verify_token};
pub use models::User;
use sqlx::PgPool;
use utils::{DecodingKey, EncodingKey};

use std::{ops::Deref, sync::Arc};

use axum::{
    middleware::from_fn_with_state,
    routing::{get, patch, post},
    Router,
};

#[derive(Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: sqlx::PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route("/users", get(list_chat_user_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let router = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state.clone());

    Ok(set_layer(router))
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pool,
            }),
        })
    }
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod test_util {
    use std::path::Path;

    use sqlx_db_tester::TestPg;

    use super::*;

    impl AppState {
        pub async fn new_for_test(config: AppConfig) -> Result<(TestPg, Self), AppError> {
            let dk = DecodingKey::load(&config.auth.pk).context("load dk failed")?;
            let ek = EncodingKey::load(&config.auth.sk).context("load ek failed")?;
            let server_url = config.server.db_url.split('/').nth(2).unwrap();
            let (tdb, pool) =
                get_test_pool(Some(format!("postgres://{server_url}").as_str())).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    dk,
                    ek,
                    pool,
                }),
            };
            Ok((tdb, state))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:postgres@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        (tdb, pool)
    }
}
