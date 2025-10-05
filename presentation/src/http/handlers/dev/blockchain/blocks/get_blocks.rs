use crate::dtos::block::BlockPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::blockchain::blocks::{
    GetBlockchainBlocksByHeightRangeUseCaseRequest, GetBlockchainBlocksByHeightRangeUseCaseResponse,
};
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Retrieves all blocks in the specified height range.
#[utoipa::path(
    tag = "Development / Blockchain",
    get,
    path = "/",
    params(GetBlockchainBlocksByHeightRangeHttpRequestQuery),
    responses(
        (status = 200, description = "Success", body = GetBlockchainBlocksByHeightRangeHttpResponseBody),
        (status = 500, description = "Failure"),
    ),
)]
pub(crate) async fn get_blocks(
    State(state): State<AppState>,
    Query(query): Query<GetBlockchainBlocksByHeightRangeHttpRequestQuery>,
) -> Result<Json<GetBlockchainBlocksByHeightRangeHttpResponseBody>, PresentationError> {
    let AppState {
        get_blockchain_blocks_by_height_range_use_case,
        ..
    } = state;
    let request = GetBlockchainBlocksByHeightRangeUseCaseRequest {
        height_range: query.start.into()..=query.end.into(),
    };
    let uc_res = get_blockchain_blocks_by_height_range_use_case
        .execute(request)
        .await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct GetBlockchainBlocksByHeightRangeHttpRequestQuery {
    /// Start block height (inclusive)
    start: u64,
    /// End block height (inclusive)
    end: u64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetBlockchainBlocksByHeightRangeHttpResponseBody {
    blocks: Vec<BlockPresentationDto>,
}

impl From<GetBlockchainBlocksByHeightRangeUseCaseResponse>
    for GetBlockchainBlocksByHeightRangeHttpResponseBody
{
    fn from(res: GetBlockchainBlocksByHeightRangeUseCaseResponse) -> Self {
        let blocks = res.blocks.into_iter().map(|block| block.into()).collect();
        Self { blocks }
    }
}
