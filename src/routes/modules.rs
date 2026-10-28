use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::Cookies;
use uuid::Uuid;
use crate::error::AppError;

#[derive(Serialize)]
pub struct ModuleStatus {
    pub slug: String,
    pub name: String,
    pub is_enabled: bool,
}

#[derive(Deserialize)]
pub struct ToggleModulePayload {
    pub module_slug: String,
    pub is_enabled: bool,
}

async fn get_authenticated_user_id(cookies: &Cookies, pool: &PgPool) -> Result<Uuid, AppError> {
    let cookie = cookies.get("session_token").ok_or(AppError::Unauthorized)?;
    let session = sqlx::query!(
        "SELECT user_id FROM user_sessions WHERE token_hash = $1 AND expires_at > NOW()",
        cookie.value()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::InternalServerError)?
    .ok_or(AppError::Unauthorized)?;

    Ok(session.user_id)
}

pub async fn list_modules(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<ModuleStatus>>, AppError> {
    let user_id = get_authenticated_user_id(&cookies, &pool).await?;

    let modules = sqlx::query_as!(
        ModuleStatus,
        r#"
        SELECT 
            m.slug, 
            m.name, 
            COALESCE(um.is_enabled, false) AS "is_enabled!"
        FROM available_modules m
        LEFT JOIN user_modules um ON m.slug = um.module_slug AND um.user_id = $1
        ORDER BY m.name ASC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| AppError::InternalServerError)?;

    Ok(Json(modules))
}

pub async fn toggle_module(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<ToggleModulePayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = get_authenticated_user_id(&cookies, &pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO user_modules (user_id, module_slug, is_enabled)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, module_slug)
        DO UPDATE SET is_enabled = EXCLUDED.is_enabled
        "#,
        user_id,
        payload.module_slug,
        payload.is_enabled
    )
    .execute(&pool)
    .await
    .map_err(|_| AppError::BadRequest("Module inexistant ou erreur d'enregistrement".into()))?;

    Ok(Json(serde_json::json!({ "status": "updated" })))
}