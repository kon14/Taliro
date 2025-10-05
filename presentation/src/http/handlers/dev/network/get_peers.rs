use crate::auth::MasterKeyAuthContextExtractor;
use crate::dtos::network::NetworkAddressPresentationDto;
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::network::GetNetworkPeersUseCaseResponse;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[allow(unused)]
use serde_json::json;

/// Retrieves the network's connected peers.
#[utoipa::path(
    tag = "Development / Network",
    get,
    path = "/peers",
    responses(
        (status = 200, description = "Success", body = GetNetworkPeersHttpResponseBody),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failure"),
    ),
    security(
        ("bearerAuth" = [])
    ),
)]
pub(crate) async fn get_peers(
    State(state): State<AppState>,
    _: MasterKeyAuthContextExtractor,
) -> Result<Json<GetNetworkPeersHttpResponseBody>, PresentationError> {
    let AppState {
        get_network_peers_use_case,
        ..
    } = state;

    let uc_res = get_network_peers_use_case.execute().await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetNetworkPeersHttpResponseBody {
    #[schema(example = json!([
        {
            "address": "/ip4/172.30.128.1/tcp/52482/p2p/12D3KooWKwUzXLNEAF97yuvyvWNVVunxAULArPj7pHWAvSveU1rc",
            "peer_id": "12D3KooWKwUzXLNEAF97yuvyvWNVVunxAULArPj7pHWAvSveU1rc"
        }
    ]))]
    peers: Vec<NetworkAddressPresentationDto>,
}

impl From<GetNetworkPeersUseCaseResponse> for GetNetworkPeersHttpResponseBody {
    fn from(res: GetNetworkPeersUseCaseResponse) -> Self {
        Self {
            peers: res.peers.into_iter().map(|addr| addr.into()).collect(),
        }
    }
}
