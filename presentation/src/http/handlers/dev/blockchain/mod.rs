use application::state::AppState;
use utoipa::OpenApi;

mod blocks;
mod get_tip_info;

use blocks::*;
use get_tip_info::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Blockchain
        get_tip_info,
    ),
    nest(
        (path = "/blocks", api = DevelopmentBlockchainBlocksApiDoc),
    ),
    tags(
        (name = "Development / Blockchain"),
    )
)]
pub struct DevelopmentBlockchainApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            &format!("{base_path}/tip"),
            axum::routing::get(get_tip_info),
        )
        .merge(blocks::declare_routes(&format!("{base_path}/blocks")))
}
