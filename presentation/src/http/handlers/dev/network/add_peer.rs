use crate::auth::MasterKeyAuthContextExtractor;
use crate::dtos::network::AddPeerResponseStatusPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use axum::extract::State;
use axum::Json;
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
        add_network_peer_use_case,
        ..
    } = store;

    let multiaddr_str = payload.network_address;
    let res = add_network_peer_use_case.execute(multiaddr_str).await?;
    let http_res = res.into();

    Ok(Json(http_res))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct AddNetworkPeerHttpRequestBody {
    /// The multiaddr of the peer to connect to.
    #[schema(
        example = "/ip4/0.0.0.0/tcp/54244/p2p/12D3KooWDGj8psQG6RjCkSaNQXFy8iYMP2UcCLz1G4GhzKUnXTAx"
    )]
    network_address: String,
}

type GetNetworkAddPeerHttpResponseBody = AddPeerResponseStatusPresentationDto;
