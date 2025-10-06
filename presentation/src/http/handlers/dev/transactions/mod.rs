use application::state::AppState;
use utoipa::OpenApi;

mod mempool;
mod place_transaction;
mod utxo;

use mempool::*;
use place_transaction::*;
use utxo::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Transactions
        place_unconfirmed_transaction,
    ),
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
        .route(
            &format!("{base_path}"),
            axum::routing::post(place_unconfirmed_transaction),
        )
        .merge(mempool::declare_routes(&format!("{base_path}/mempool")))
        .merge(utxo::declare_routes(&format!("{base_path}/utxo")))
}
