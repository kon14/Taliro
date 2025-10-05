use application::state::AppState;
use utoipa::OpenApi;

mod get_utxos;

use get_utxos::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_utxos,
    ),
    tags(
        (name = "Development / Transactions"),
    )
)]
pub struct DevelopmentTransactionsUtxoApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(&format!("{base_path}"), axum::routing::get(get_utxos))
}
