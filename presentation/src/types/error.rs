use axum::http::StatusCode;
use common::error::AppError;
use serde_json::json;

/// A wrapper type extending [`common::error::AppError`] with presentation layer semantics.<br />
/// Used to implement [`axum::response::IntoResponse`] for [`common::error::AppError`].<br />
pub(crate) struct PresentationError(AppError);

impl From<AppError> for PresentationError {
    fn from(err: AppError) -> Self {
        PresentationError(err)
    }
}

impl PresentationError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            // Note: Consider propagating a source flag for some of these in the future.
            AppError::BlockValidation(_) => StatusCode::BAD_REQUEST,
            AppError::TransactionValidation(_) => StatusCode::BAD_REQUEST,
            AppError::ConsensusValidation(_) => StatusCode::BAD_REQUEST,
            AppError::Cryptographic(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Network(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::PreconditionFailed(_) => StatusCode::PRECONDITION_FAILED,
        }
    }
}

impl axum::response::IntoResponse for PresentationError {
    fn into_response(self) -> axum::response::Response {
        self.0.to_string();

        let body = axum::Json(json!({
            "error": {
                "type": self.0.get_error_type(),
                "message": self.0.get_public_info(),
            }
        }));
        (self.status_code(), body).into_response()
    }
}
