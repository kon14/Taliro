use application::state::AppState;
use utoipa::OpenApi;

mod get_block;
mod get_blocks;
mod mine;

use get_block::*;
use get_blocks::*;
use mine::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_block,
        get_blocks,
        mine_block,
    ),
    tags(
        (name = "Development / Blockchain"),
    )
)]
pub struct DevelopmentBlockchainBlocksApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            &format!("{base_path}/{{block_hash}}"),
            axum::routing::get(get_block),
        )
        .route(&format!("{base_path}"), axum::routing::get(get_blocks))
        .route(
            &format!("{base_path}/mine"),
            axum::routing::post(mine_block),
        )
}
