use crate::auth::MasterKeyAuthContextExtractor;
use crate::conversions::time::DateTimeExtPresentation;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::InitiateGenesisUseCaseRequest;
use axum::extract::State;
use axum::Json;
use chrono::{DateTime, Utc};
use common::error::AppError;
use domain::entities::transaction::TransactionAmount;
use domain::genesis::config::{GenesisConfig, GenesisConfigUtxoFunds};
use domain::types::sign::PublicKey;
use serde::Deserialize;
use std::str::FromStr;
use utoipa::ToSchema;

#[allow(unused)]
use serde_json::json;

/// Bootstraps the blockchain by seeding it with a genesis block.
#[utoipa::path(
    tag = "Development",
    post,
    path = "/genesis",
    responses(
        (status = 200, description = "Success", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failure"),
    ),
    security(
        ("bearerAuth" = [])
    ),
)]
pub(crate) async fn init_genesis(
    State(state): State<AppState>,
    _: MasterKeyAuthContextExtractor,
    Json(payload): Json<InitiateGenesisHttpRequestBody>,
) -> Result<String, PresentationError> {
    let AppState {
        init_genesis_use_case,
        ..
    } = state;

    let uc_req = payload.try_into()?;
    init_genesis_use_case.execute(uc_req).await?;

    Ok("Blockchain bootstrapped successfully!".to_string())
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct InitiateGenesisHttpRequestBody {
    #[schema(example = json!({
        "utxos": [
            {
                "wallet_pub_key": "59f783b83cf3b6552f53044743ac3454a84ed9b47897ef1576e64662363dbd6b",
                "amount": 1000000000u128
            }
        ],
        "timestamp": "2025-09-08T00:00:00Z"
    }))]
    genesis_cfg: GenesisConfigHttp,
}

#[derive(Deserialize, ToSchema)]
struct GenesisConfigHttp {
    /// List of unspent transaction outputs for the genesis block.
    utxos: Vec<GenesisConfigUtxoFundsHttp>,

    /// Timestamp for the genesis configuration, in ISO8601 UTC.
    #[schema(format = "date-time", example = "2025-09-08T12:34:56Z")]
    timestamp: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
struct GenesisConfigUtxoFundsHttp {
    /// Wallet public key where the amount should be deposited.
    #[schema(example = "59f783b83cf3b6552f53044743ac3454a84ed9b47897ef1576e64662363dbd6b")]
    wallet_pub_key: String,

    /// Transaction amount.
    #[schema(example = 1000000000)]
    amount: u128,
}

impl TryFrom<InitiateGenesisHttpRequestBody> for InitiateGenesisUseCaseRequest {
    type Error = AppError;

    fn try_from(req: InitiateGenesisHttpRequestBody) -> Result<Self, Self::Error> {
        let req = Self {
            genesis_cfg: req.genesis_cfg.try_into()?,
        };
        Ok(req)
    }
}

impl TryFrom<GenesisConfigHttp> for GenesisConfig {
    type Error = AppError;

    fn try_from(req: GenesisConfigHttp) -> Result<Self, Self::Error> {
        let utxos = req
            .utxos
            .into_iter()
            .map(|funds| funds.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        let timestamp = req.timestamp.to_dmn_datetime();
        let cfg = Self::new_unchecked(utxos, timestamp);
        Ok(cfg)
    }
}

impl TryFrom<GenesisConfigUtxoFundsHttp> for GenesisConfigUtxoFunds {
    type Error = AppError;

    fn try_from(data: GenesisConfigUtxoFundsHttp) -> Result<Self, Self::Error> {
        let public_key = PublicKey::from_str(&data.wallet_pub_key)?;
        let amount = TransactionAmount::new(data.amount);
        let funds = Self::new_unchecked(public_key, amount);
        Ok(funds)
    }
}
