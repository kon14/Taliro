use crate::dtos::transaction::UtxoPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::transactions::utxo::GetUtxosUseCaseResponse;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Retrieves all UTXOs.
#[utoipa::path(
    tag = "Development / Transactions",
    get,
    path = "/",
    responses(
        (status = 200, description = "Success", body = GetUtxosHttpResponseBody),
        (status = 500, description = "Failure"),
    ),
)]
pub(crate) async fn get_utxos(
    State(state): State<AppState>,
) -> Result<Json<GetUtxosHttpResponseBody>, PresentationError> {
    let AppState {
        get_utxos_use_case, ..
    } = state;
    let uc_res = get_utxos_use_case.execute().await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetUtxosHttpResponseBody {
    utxos: Vec<UtxoPresentationDto>,
}

impl From<GetUtxosUseCaseResponse> for GetUtxosHttpResponseBody {
    fn from(res: GetUtxosUseCaseResponse) -> Self {
        let utxos = res.utxos.into_iter().map(|utxo| utxo.into()).collect();
        Self { utxos }
    }
}
