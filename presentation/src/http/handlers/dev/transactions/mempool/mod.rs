use application::state::AppState;
use utoipa::OpenApi;

mod get_transactions;
mod place_transaction;

use get_transactions::*;
use place_transaction::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_transactions,
        place_transaction,
    ),
    tags(
        (name = "Development / Transactions"),
    )
)]
pub struct DevelopmentTransactionsMempoolApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            &format!("{base_path}"),
            axum::routing::get(get_transactions),
        )
        .route(
            &format!("{base_path}"),
            axum::routing::post(place_transaction),
        )
}
