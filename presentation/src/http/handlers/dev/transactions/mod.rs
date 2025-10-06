use application::state::AppState;
use utoipa::OpenApi;

mod mempool;
mod utxo;

use mempool::*;
use utxo::*;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/mempool", api = DevelopmentTransactionsMempoolApiDoc),
        (path = "/utxo", api = DevelopmentTransactionsUtxoApiDoc),
    ),
    tags(
        (name = "Development / Transactions"),
    )
)]
pub struct DevelopmentTransactionsApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .merge(mempool::declare_routes(&format!("{base_path}/mempool")))
        .merge(utxo::declare_routes(&format!("{base_path}/utxo")))
}
