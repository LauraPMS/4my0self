use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::{
    auth::password::{hash_password, verify_password},
    error::AppError,
};

#[derive(Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub email: String,
}

pub async fn register(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    if payload.password.len() < 12 {
        return Err(AppError::BadRequest("Le mot de passe doit contenir au moins 12 caractères".into()));
    }

    let password_hash = hash_password(&payload.password)?;

    let record = sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, email",
        payload.email,
        password_hash
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| match e.as_database_error().and_then(|db_err| db_err.code()) {
        Some(code) if code == "23505" => AppError::Conflict("Cet email est déjà utilisé".into()),
        _ => AppError::InternalServerError,
    })?;

    Ok(Json(AuthResponse {
        user_id: record.id,
        email: record.email,
    }))
}

pub async fn login(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query!(
        "SELECT id, email, password_hash FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| AppError::InternalServerError)?
    .ok_or(AppError::Unauthorized)?;

    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    // Création d'une session stateful
    let raw_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query!(
        "INSERT INTO user_sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        raw_token,
        expires_at
    )
    .execute(&pool)
    .await
    .map_err(|_| AppError::InternalServerError)?;

    // Cookie durci HttpOnly, SameSite=Strict
    let mut cookie = Cookie::new("session_token", raw_token);
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok(Json(AuthResponse {
        user_id: user.id,
        email: user.email,
    }))
}

pub async fn logout(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(cookie) = cookies.get("session_token") {
        let token = cookie.value();
        let _ = sqlx::query!("DELETE FROM user_sessions WHERE token_hash = $1", token)
            .execute(&pool)
            .await;
        cookies.remove(Cookie::from("session_token"));
    }
    Ok(Json(serde_json::json!({ "message": "Déconnexion réussie" })))
}