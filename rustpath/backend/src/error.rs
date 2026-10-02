use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Уровень пока закрыт. Завершите предыдущий.")]
    Locked,
    #[error("Сессия истекла. Обновите страницу.")]
    Unauthorized,
    #[error("Уровень не найден.")]
    NotFound,
    #[error("Слишком много попыток. Подождите минуту.")]
    RateLimited,
    #[error("Проверка временно недоступна. Попробуйте ещё раз.")]
    Runner,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Locked => StatusCode::FORBIDDEN,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Runner => StatusCode::SERVICE_UNAVAILABLE,
            Self::Database(e) => {
                tracing::error!(error = %e, "database failure");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let message = if matches!(self, Self::Database(_)) {
            "Внутренняя ошибка. Повторите позже.".to_owned()
        } else {
            self.to_string()
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}
