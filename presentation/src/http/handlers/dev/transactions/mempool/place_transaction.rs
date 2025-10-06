use crate::auth::MasterKeyAuthContextExtractor;
use crate::dtos::transaction::{TransactionOutPointPresentationDto, TransactionPresentationDto};
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::transactions::mempool::{
    PlaceMempoolTransactionUseCaseRequest, PlaceMempoolTransactionUseCaseResponse,
};
use axum::extract::State;
use axum::Json;
use common::error::AppError;
use domain::entities::transaction::{TransactionAmount, TransactionOutPoint};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Places a new transaction in the mempool.
/// Accepts a wallet's private signing key for convenience.
#[utoipa::path(
    tag = "Development / Transactions",
    post,
    path = "/",
    responses(
        (status = 200, description = "Success", body = PlaceMempoolTransactionHttpResponseBody),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failure"),
    ),
    security(
        ("bearerAuth" = [])
    ),
)]
pub(crate) async fn place_transaction(
    State(state): State<AppState>,
    _: MasterKeyAuthContextExtractor,
    Json(payload): Json<PlaceMempoolTransactionHttpRequestBody>,
) -> Result<Json<PlaceMempoolTransactionHttpResponseBody>, PresentationError> {
    let AppState {
        place_mempool_transaction_use_case,
        ..
    } = state;

    let uc_req = payload.try_into()?;
    let uc_res = place_mempool_transaction_use_case.execute(uc_req).await?;
    let http_res: PlaceMempoolTransactionHttpResponseBody = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct PlaceMempoolTransactionHttpRequestBody {
    /// The sender wallet's private signing key.<br />
    /// Used for convenience in dev environment.
    #[schema(example = "ee593f0203f6e97c9ce47c6a8f15582a48635cb0354f515c1944c881091914a7")]
    sender_private_key: String,

    /// The recipient wallet's address.
    #[schema(example = "54b73c091395a30874a397cbfcd54c7348175a01ee6ccf0a1133f8f8b3a19e7d")]
    recipient_wallet_address: String,

    /// The amount to be transferred.<br />
    /// Remaining change is sent back to sender.
    #[schema(example = 500u128)]
    amount: u128,

    /// The outpoints to be consumed as inputs for this transaction.
    #[schema(example = json!([
        {
            "tx_id": "7a35a28e484dd0cf825c036685fa8f910f122f3b27910185ea6147ef7814442f",
            "tx_output_index": 0,
        }
    ]))]
    consumed_outpoints: Vec<TransactionOutPointPresentationDto>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PlaceMempoolTransactionHttpResponseBody {
    transaction: TransactionPresentationDto,
}

impl TryFrom<PlaceMempoolTransactionHttpRequestBody> for PlaceMempoolTransactionUseCaseRequest {
    type Error = AppError;

    fn try_from(req: PlaceMempoolTransactionHttpRequestBody) -> Result<Self, Self::Error> {
        let sender_private_key = req.sender_private_key.parse()?;
        let recipient_wallet_address = req.recipient_wallet_address.parse()?;
        let consumed_outpoints = req
            .consumed_outpoints
            .into_iter()
            .map(|op| op.try_into())
            .collect::<Result<Vec<TransactionOutPoint>, AppError>>()?;
        let dmn_req = Self {
            sender_private_key,
            recipient_wallet_address,
            amount: TransactionAmount::new(req.amount),
            consumed_outpoints,
        };
        Ok(dmn_req)
    }
}

impl From<PlaceMempoolTransactionUseCaseResponse> for PlaceMempoolTransactionHttpResponseBody {
    fn from(res: PlaceMempoolTransactionUseCaseResponse) -> Self {
        Self {
            transaction: res.transaction.into(),
        }
    }
}
