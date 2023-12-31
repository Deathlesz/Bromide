use axum::{extract::State, Form, response::IntoResponse};
use sqlx::query;

use crate::{
    AppState,
    error::{LoginError, RegisterError}, forms, utils,
};

pub(super) async fn register(
    State(state): State<AppState>,
    Form(payload): Form<forms::accounts::RegisterGJAccount>,
) -> Result<impl IntoResponse, RegisterError> {
    if !payload.user_name.chars().all(char::is_alphanumeric) {
        return Err(RegisterError::UserNameIsNotAlphanumeric);
    } else if payload.user_name.len() < 3 {
        return Err(RegisterError::UserNameTooShort);
    } else if payload.user_name.len() > 20 {
        return Err(RegisterError::UserNameTooLong);
    } else if !payload
        .password
        .chars()
        .all(|char| char.is_alphanumeric() || char == '-' || char == '_')
    {
        return Err(RegisterError::InvalidPassword);
    } else if payload.password.len() < 3 {
        return Err(RegisterError::PasswordTooShort);
    } else if payload.password.len() > 20 {
        return Err(RegisterError::PasswordTooLong);
    } else if !email_address::EmailAddress::is_valid(&payload.email) {
        return Err(RegisterError::InvalidEmail);
    }

    let count = query!(
        "SELECT COUNT(1) as count FROM `accounts` WHERE `user_name` = ?",
        payload.user_name
    )
        .fetch_one(state.pool())
        .await?
        .count;

    if count > 0 {
        return Err(RegisterError::UserNameIsTaken);
    }

    let count = query!(
        "SELECT COUNT(1) as count FROM `accounts` WHERE `email` = ?",
        payload.email
    )
        .fetch_one(state.pool())
        .await?
        .count;

    if count > 0 {
        return Err(RegisterError::EmailIsTaken);
    }

    let hash = utils::password_hash(&payload.password);

    // using transaction here because we don't want to create an account if user creation is failed to create and vice versa
    let mut transcation = state.pool().begin().await?;
    let account_id = query!(
        "INSERT INTO `accounts` (`user_name`, `email`, `password`) VALUES (?, ?, ?) RETURNING `id`",
        payload.user_name,
        payload.email,
        hash
    )
        .fetch_one(&mut *transcation)
        .await?
        .id;

    query!("INSERT INTO `users` (`account_id`) VALUES (?)", account_id)
        .execute(&mut *transcation)
        .await?;

    // if function errored then transaction will go out of scope and will be rolled back according to docs
    // otherwise, commit
    transcation.commit().await?;

    Ok("1")
}

pub(crate) async fn login(
    State(state): State<AppState>,
    Form(payload): Form<forms::accounts::LoginGJAccount>,
) -> Result<impl IntoResponse, LoginError> {
    let result = query!(
        "SELECT `id`, `password` FROM `accounts` WHERE `user_name` = ?",
        payload.user_name
    )
        .fetch_one(state.pool())
        .await?;

    if result.password != payload.gjp2 {
        return Err(LoginError::IncorrectCredentials);
    }

    let user_id = query!("SELECT `id` FROM `users` WHERE `account_id` = ?", result.id)
        .fetch_one(state.pool())
        .await?
        .id;

    Ok(format!("{},{}", result.id, user_id))
}
