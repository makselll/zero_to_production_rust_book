use actix_web::{web, HttpResponse, Responder};
use actix_web::web::Form;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use uuid::Uuid;
use crate::damain::{NewSubscriber, SubscriberEmail};
use crate::damain::SubscriberName;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        Ok(NewSubscriber{
            email: SubscriberEmail::parse(form.email)?,
            name: SubscriberName::parse(form.name)?
        })
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, db_pool),
)]
pub async fn insert_subscriber(new_subscriber: &NewSubscriber, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO
            subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)", Uuid::new_v4(), new_subscriber.name.as_ref(), new_subscriber.email.as_ref(), Utc::now()
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
pub async fn subscriptions(Form(form): Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let new_subscriber = match form.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(e) => return HttpResponse::BadRequest()
    };
    
    match insert_subscriber(&new_subscriber, &db_pool).await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError()
    }
}


