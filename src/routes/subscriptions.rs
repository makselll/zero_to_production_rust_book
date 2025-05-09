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

pub async fn subscriptions(form: Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query!(
        "INSERT INTO
            subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)", Uuid::new_v4(), form.name, form.email, Utc::now()
    ).execute(db_pool.get_ref()).await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError()
        }
    }
}

