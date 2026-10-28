use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    InternalServerError,
    Unauthorized,
    BadRequest(String),
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Erreur interne du serveur"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Accès non autorisé"),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.as_str()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}