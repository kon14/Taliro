use application::state::AppState;
use utoipa::OpenApi;

mod get_transactions;

use get_transactions::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_transactions,
    ),
    tags(
        (name = "Development / Transactions"),
    )
)]
pub struct DevelopmentTransactionsMempoolApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(
        &format!("{base_path}"),
        axum::routing::get(get_transactions),
    )
}
