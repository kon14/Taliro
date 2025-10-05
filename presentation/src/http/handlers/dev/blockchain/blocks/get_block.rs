use crate::dtos::block::BlockPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::blockchain::blocks::{
    GetBlockchainBlockUseCaseRequest, GetBlockchainBlockUseCaseResponse,
};
use axum::extract::{Path, State};
use axum::Json;
use domain::types::hash::Hash;
use serde::Serialize;
use utoipa::ToSchema;

/// Retrieves a block by its hash.
#[utoipa::path(
    tag = "Development / Blockchain",
    get,
    path = "/{block_hash}",
    params(
        ("block_hash" = String, Path),
    ),
    responses(
        (status = 200, description = "Success", body = GetBlockchainBlockHttpResponseBody),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Failure"),
    ),
)]
pub(crate) async fn get_block(
    State(state): State<AppState>,
    Path(block_hash): Path<String>,
) -> Result<Json<GetBlockchainBlockHttpResponseBody>, PresentationError> {
    let AppState {
        get_blockchain_block_use_case,
        ..
    } = state;

    let block_hash = Hash::try_from(block_hash.as_ref())?;
    let request = GetBlockchainBlockUseCaseRequest { block_hash };
    let uc_res = get_blockchain_block_use_case.execute(request).await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetBlockchainBlockHttpResponseBody {
    block: Option<BlockPresentationDto>,
}

impl From<GetBlockchainBlockUseCaseResponse> for GetBlockchainBlockHttpResponseBody {
    fn from(res: GetBlockchainBlockUseCaseResponse) -> Self {
        let block = res.block.map(|block| block.into());
        Self { block }
    }
}
