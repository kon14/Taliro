use crate::types::error::PresentationError;
use application::auth::master_key::MasterKeyAuthContext;
use application::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use common::error::AppError;

#[allow(unused)]
pub(crate) struct MasterKeyAuthContextExtractor(Option<MasterKeyAuthContext>);

impl FromRequestParts<AppState> for MasterKeyAuthContextExtractor {
    type Rejection = PresentationError;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        const UNAUTHORIZED_ERR_STR: &str = "Failed to authenticate user!";

        if !app_state.master_key_authenticator.is_enabled() {
            return Ok(MasterKeyAuthContextExtractor(None));
        }

        async fn get_ctx(
            parts: &mut Parts,
            app_state: &AppState,
        ) -> Result<Option<MasterKeyAuthContext>, AppError> {
            let Some(master_key) = extract_master_key_from_headers(parts).await? else {
                return Ok(None);
            };
            let ctx = app_state
                .master_key_authenticator
                .authenticate_master_key(master_key)?;
            Ok(Some(ctx))
        }

        // Obfuscate error specifics for security.
        get_ctx(parts, app_state)
            .await
            .map(|ctx| MasterKeyAuthContextExtractor(ctx))
            .map_err(|err| {
                let reworded = match err {
                    AppError::Unauthorized(base) => {
                        AppError::Unauthorized(base.reword(UNAUTHORIZED_ERR_STR))
                    }
                    err => err,
                };
                reworded.into()
            })
    }
}

async fn extract_master_key_from_headers(parts: &mut Parts) -> Result<Option<String>, AppError> {
    const UNAUTHORIZED_ERR_STR: &str = "Failed to extract authentication master key secret!";

    match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
        Ok(TypedHeader(Authorization(bearer))) => {
            let token = bearer.token().trim();
            if token.is_empty() {
                Ok(None)
            } else {
                Ok(Some(token.to_string()))
            }
        }
        Err(err) if err.is_missing() => Ok(None),
        Err(err) => Err(AppError::unauthorized_with_private(
            UNAUTHORIZED_ERR_STR,
            err.to_string(),
        )),
    }
}
