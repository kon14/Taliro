mod blockchain;
mod generate_wallet;
mod genesis;
mod network;
mod transactions;

use application::state::AppState;
use blockchain::*;
use generate_wallet::*;
use genesis::*;
use network::*;
use transactions::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Development
        generate_wallet,
        init_genesis,
    ),
    nest(
        (path = "/network", api = DevelopmentNetworkApiDoc),
        (path = "/blockchain", api = DevelopmentBlockchainApiDoc),
        (path = "/transactions", api = DevelopmentTransactionsApiDoc),
    ),
    tags(
        (name = "Development"),
    )
)]
pub struct DevelopmentApiDoc;

pub fn declare_routes(base_path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            &format!("{base_path}/generate-wallet"),
            axum::routing::post(generate_wallet),
        )
        .route(
            &format!("{base_path}/genesis"),
            axum::routing::post(init_genesis),
        )
        .merge(network::declare_routes(&format!("{base_path}/network")))
        .merge(blockchain::declare_routes(&format!(
            "{base_path}/blockchain"
        )))
        .merge(transactions::declare_routes(&format!(
            "{base_path}/transactions"
        )))
}
