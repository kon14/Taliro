use crate::auth::MasterKeyAuthContextExtractor;
use crate::dtos::network::AddPeerResponseStatusPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use axum::extract::State;
use axum::Json;
use common::log_http_error;
use serde::Deserialize;
use utoipa::ToSchema;

/// Connects to a node.
#[utoipa::path(
    tag = "Development / Network",
    post,
    path = "/peers",
    responses(
        (status = 200, description = "Success", body = GetNetworkAddPeerHttpResponseBody),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failure"),
    ),
    security(
        ("bearerAuth" = [])
    ),
)]
pub(crate) async fn add_peer(
    State(store): State<AppState>,
    _: MasterKeyAuthContextExtractor,
    Json(payload): Json<AddNetworkPeerHttpRequestBody>,
) -> Result<Json<GetNetworkAddPeerHttpResponseBody>, PresentationError> {
    let AppState {
        net_entity_validator,
        add_network_peer_use_case,
        ..
    } = store;

    // Validate Network Address
    let network_address = match net_entity_validator.validate_address(payload.network_address) {
        Ok(addr) => addr,
        Err(err) => {
            log_http_error!("{err}");
            return Err(err.into());
        }
    };

    let res = add_network_peer_use_case.execute(network_address).await?;
    let http_res = res.into();

    Ok(Json(http_res))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct AddNetworkPeerHttpRequestBody {
    /// The multiaddr of the peer to connect to.<br />
    /// Must include a P2P peer ID!
    #[schema(
        example = "/ip4/0.0.0.0/tcp/54244/p2p/12D3KooWDGj8psQG6RjCkSaNQXFy8iYMP2UcCLz1G4GhzKUnXTAx"
    )]
    network_address: String,
}

type GetNetworkAddPeerHttpResponseBody = AddPeerResponseStatusPresentationDto;
