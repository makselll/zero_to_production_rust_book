use actix_web::{web, HttpResponse, Responder};
use actix_web::web::Form;
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use sqlx::types::chrono::Utc;
use uuid::Uuid;
use crate::damain::{NewSubscriber, SubscriberEmail};
use crate::damain::SubscriberName;
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;

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

fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction),
)]
pub async fn insert_subscriber(new_subscriber: &NewSubscriber, transaction: &mut Transaction<'_, Postgres>,) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO
            subscriptions (id, name, email, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')", subscriber_id, new_subscriber.name.as_ref(), new_subscriber.email.as_ref(), Utc::now()
    )
        .execute(&mut **transaction)
        .await
        .map_err(|e| {tracing::error!("Failed to execute query: {:?}", e); e})?;
    Ok(subscriber_id)
}


#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?token={}", base_url, token)
    ;
    email_client.send_email(
        new_subscriber.email,
        "Welcome!",
        &format!(
            "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
            confirmation_link
        ), &format!(
            "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
            confirmation_link
        )
    ).await
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(token, transaction),
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        token,
        subscriber_id
    )
        .execute(&mut **transaction)
        .await
        .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e); e })?;
    Ok(())
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]

pub async fn subscriptions(
    Form(form): Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>
) -> impl Responder {
    let mut transaction = match db_pool.begin().await {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError()
    };
    let new_subscriber = match form.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest()
    };
    let token = generate_subscription_token();
    
    let subscriber_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError()
    };
    if store_token(&mut transaction, subscriber_id, &token).await.is_err() {
        return HttpResponse::InternalServerError();
    };
    
    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError();   
    }
    
    if send_confirmation_email(&email_client, new_subscriber, &base_url.0, &token).await.is_err() {
        return HttpResponse::InternalServerError();
    }

    HttpResponse::Ok()
}


