use crate::auth::MasterKeyAuthContextExtractor;
use crate::dtos::block::BlockPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::blockchain::blocks::{
    AdHocMineBlockUseCaseRequest, AdHocMineBlockUseCaseResponse,
};
use axum::extract::State;
use axum::Json;
use common::error::AppError;
use domain::types::hash::Hash;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Initiates ad-hoc block mining.
#[utoipa::path(
    tag = "Development / Blockchain",
    post,
    path = "/mine",
    responses(
        (status = 200, description = "Success", body = AdHocMineBlockHttpResponseBody),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failure"),
    ),
    security(
        ("bearerAuth" = [])
    ),
)]
pub(crate) async fn mine_block(
    State(state): State<AppState>,
    _: MasterKeyAuthContextExtractor,
    Json(payload): Json<AdHocMineBlockHttpRequestBody>,
) -> Result<Json<AdHocMineBlockHttpResponseBody>, PresentationError> {
    let AppState {
        adhoc_mine_block_use_case,
        ..
    } = state;

    let uc_req = payload.try_into()?;
    let uc_res = adhoc_mine_block_use_case.execute(uc_req).await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct AdHocMineBlockHttpRequestBody {
    transaction_hashes: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct AdHocMineBlockHttpResponseBody {
    block: BlockPresentationDto,
}

impl From<AdHocMineBlockUseCaseResponse> for AdHocMineBlockHttpResponseBody {
    fn from(res: AdHocMineBlockUseCaseResponse) -> Self {
        let block = res.block.into();
        Self { block }
    }
}

impl TryFrom<AdHocMineBlockHttpRequestBody> for AdHocMineBlockUseCaseRequest {
    type Error = AppError;

    fn try_from(req: AdHocMineBlockHttpRequestBody) -> Result<Self, Self::Error> {
        let transaction_hashes = req
            .transaction_hashes
            .into_iter()
            .map(|tx_hash| Hash::try_from(tx_hash.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        let request = Self { transaction_hashes };
        Ok(request)
    }
}
