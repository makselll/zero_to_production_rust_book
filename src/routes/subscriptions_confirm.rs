use actix_web::{web, HttpResponse, Responder};
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
pub async fn subscriptions_confirm(web::Query(token): web::Query<Token>, db_pool: web::Data<PgPool>) -> impl Responder {
    let subscriber_id = match get_subscriber_id(token.token, &db_pool).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest()
    };
    match update_subscriber(subscriber_id, &db_pool).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError()
    }
}