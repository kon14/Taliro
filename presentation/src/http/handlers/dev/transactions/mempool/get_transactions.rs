use crate::dtos::params::PaginationParamsPresentationDto;
use crate::dtos::transaction::TransactionPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::transactions::mempool::GetMempoolTransactionsUseCaseResponse;
use axum::extract::{Query, State};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Retrieves all mempool transactions.
#[utoipa::path(
    tag = "Development / Transactions",
    get,
    path = "/",
    params(
        PaginationParamsPresentationDto,
    ),
    responses(
        (status = 200, description = "Success", body = GetMempoolTransactionsHttpResponseBody),
        (status = 500, description = "Failure"),
    ),
)]
pub(crate) async fn get_transactions(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParamsPresentationDto>,
) -> Result<Json<GetMempoolTransactionsHttpResponseBody>, PresentationError> {
    let AppState {
        get_mempool_transactions_use_case,
        ..
    } = state;
    let pagination = pagination.into();
    let uc_res = get_mempool_transactions_use_case
        .execute(pagination)
        .await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetMempoolTransactionsHttpResponseBody {
    transactions: Vec<TransactionPresentationDto>,
    count: usize,
}

impl From<GetMempoolTransactionsUseCaseResponse> for GetMempoolTransactionsHttpResponseBody {
    fn from(res: GetMempoolTransactionsUseCaseResponse) -> Self {
        let transactions = res.transactions.into_iter().map(|tx| tx.into()).collect();
        Self {
            transactions,
            count: res.count,
        }
    }
}
