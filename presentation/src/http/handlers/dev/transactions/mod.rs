use application::state::AppState;
use utoipa::OpenApi;

mod place_transaction;
mod utxo;

use place_transaction::*;
use utxo::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Transactions
        place_unconfirmed_transaction,
    ),
    nest(
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
        .merge(utxo::declare_routes(&format!("{base_path}/utxo")))
}
