use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use anyhow::Context;
use serde::Deserialize;
use sqlx::PgPool;
use crate::damain::SubscriberId;


#[derive(Deserialize, Debug)]
pub struct Token {
    token: String,
}


#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(db_pool),
)]
async fn get_subscriber_id(token: String, db_pool: &PgPool) -> Result<SubscriberId, sqlx::Error> {
    let subscription_token = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1", token
    )
        .fetch_one(db_pool)
        .await
        .map_err(|e| {tracing::error!("Failed to execute query: {:?}", e); e})?;

    Ok(SubscriberId::new(subscription_token.subscriber_id))
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(db_pool),
)]
async fn update_subscriber(subscriber_id: SubscriberId, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE subscriptions SET status = 'confirmed' WHERE id = $1", subscriber_id.inner()
    )
        .execute(db_pool)
        .await
        .map_err(|e| {tracing::error!("Failed to execute query: {:?}", e); e})?;
    Ok(())
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(db_pool),
)]
pub async fn subscriptions_confirm(web::Query(token): web::Query<Token>, db_pool: web::Data<PgPool>) -> Result<HttpResponse, ConfirmError> {
    let subscriber_id = get_subscriber_id(token.token, &db_pool).await.map_err(|e| ConfirmError::InvalidToken(e.to_string()))?;
    update_subscriber(subscriber_id, &db_pool).await.context("Failed to update subscriber")?;
    Ok(HttpResponse::Ok().finish())
}


#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("{0}")]
    InvalidToken(String),
    #[error("transparent")]
    UnexpectedError(#[from] anyhow::Error)
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::routes::subscriptions::error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmError::InvalidToken(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}