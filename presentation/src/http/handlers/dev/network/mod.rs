use application::state::AppState;
use utoipa::OpenApi;

mod add_peer;
mod get_peers;
mod get_self_info;

use add_peer::*;
use get_peers::*;
use get_self_info::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Network
        get_self_info,
        get_peers,
        add_peer,
    ),
    tags(
        (name = "Development / Network"),
    )
)]
pub struct DevelopmentNetworkApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            &format!("{base_path}/self"),
            axum::routing::get(get_self_info),
        )
        .route(&format!("{base_path}/peers"), axum::routing::get(get_peers))
        .route(&format!("{base_path}/peers"), axum::routing::post(add_peer))
}
