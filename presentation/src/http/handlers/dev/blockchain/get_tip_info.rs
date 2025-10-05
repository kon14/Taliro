use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::blockchain::GetBlockchainTipInfoUseCaseResponse;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Retrieves blockchain tip information.
#[utoipa::path(
    tag = "Development / Blockchain",
    get,
    path = "/tip",
    responses(
        (status = 200, description = "Success", body = GetBlockchainTipInfoHttpResponseBody),
        (status = 500, description = "Failure"),
    ),
)]
pub(crate) async fn get_tip_info(
    State(state): State<AppState>,
) -> Result<Json<GetBlockchainTipInfoHttpResponseBody>, PresentationError> {
    let AppState {
        get_blockchain_tip_info_use_case,
        ..
    } = state;

    let uc_res = get_blockchain_tip_info_use_case.execute().await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetBlockchainTipInfoHttpResponseBody {
    block: Option<GetBlockchainTipInfoHttpResponseBodyBlock>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetBlockchainTipInfoHttpResponseBodyBlock {
    hash: String,
    height: u64,
}

impl From<GetBlockchainTipInfoUseCaseResponse> for GetBlockchainTipInfoHttpResponseBody {
    fn from(res: GetBlockchainTipInfoUseCaseResponse) -> Self {
        let block = res
            .block
            .map(|block| GetBlockchainTipInfoHttpResponseBodyBlock {
                hash: block.hash.to_string(),
                height: block.height.as_u64(),
            });
        Self { block }
    }
}
