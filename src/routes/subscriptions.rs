use actix_web::{web, HttpResponse, Responder};
use actix_web::web::Form;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, db_pool),
)]
pub async fn insert_subscriber(form: &FormData, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO
            subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)", Uuid::new_v4(), form.name, form.email, Utc::now()
    )
        .execute(db_pool)
        .await
        .map_err(|e| {tracing::error!("Failed to execute query: {:?}", e); e})?;
    Ok(())
}


#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool),
    fields(
 subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscriptions(form: Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    match insert_subscriber(&form, &db_pool).await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError()
    }
}

