use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::GenerateWalletUseCaseResponse;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Generates a new wallet with a private and public key pair.
#[utoipa::path(
    tag = "Development",
    post,
    path = "/generate-wallet",
    responses(
        (status = 200, description = "Success", body = GenerateWalletHttpResponseBody),
        (status = 500, description = "Failure"),
    ),
)]
pub(super) async fn generate_wallet(
    State(state): State<AppState>,
) -> Result<Json<GenerateWalletHttpResponseBody>, PresentationError> {
    let AppState {
        generate_wallet_use_case,
        ..
    } = state;

    let uc_res = generate_wallet_use_case.execute()?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GenerateWalletHttpResponseBody {
    #[schema(example = "ee593f0203f6e97c9ce47c6a8f15582a48635cb0354f515c1944c881091914a7")]
    private_key: String,

    #[schema(example = "59f783b83cf3b6552f53044743ac3454a84ed9b47897ef1576e64662363dbd6b")]
    public_key: String,

    #[schema(example = "e83e6364e5d6ebb0dcdc37966181b4e1b781262264d25d2dc50359c6bac1f0d9")]
    wallet_address: String,
}

impl From<GenerateWalletUseCaseResponse> for GenerateWalletHttpResponseBody {
    fn from(res: GenerateWalletUseCaseResponse) -> Self {
        Self {
            private_key: res.private_key.to_string(),
            public_key: res.public_key.to_string(),
            wallet_address: res.wallet_address.to_string(),
        }
    }
}
