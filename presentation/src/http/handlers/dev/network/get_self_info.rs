use crate::auth::MasterKeyAuthContextExtractor;
use crate::dtos::network::{NetworkAddressPresentationDto, NetworkIdentityKeypairPresentationDto};
use crate::types::error::PresentationError;
use application::state::AppState;
use application::usecases::dev::network::GetNetworkSelfInfoUseCaseResponse;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[allow(unused)]
use serde_json::json;

/// Retrieves the node's network information.
#[utoipa::path(
    tag = "Development / Network",
    get,
    path = "/self",
    responses(
        (status = 200, description = "Success", body = GetNetworkSelfInfoHttpResponseBody),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failure"),
    ),
    security(
        ("bearerAuth" = [])
    ),
)]
pub(crate) async fn get_self_info(
    State(store): State<AppState>,
    _: MasterKeyAuthContextExtractor,
) -> Result<Json<GetNetworkSelfInfoHttpResponseBody>, PresentationError> {
    let AppState {
        get_network_self_info_use_case,
        ..
    } = store;

    let uc_res = get_network_self_info_use_case.execute().await?;
    let http_res = uc_res.into();

    Ok(Json(http_res))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetNetworkSelfInfoHttpResponseBody {
    #[schema(
        example = "CAESQPDur8zTyaDoZwmCIhtpdaE5s-TjOZd8iQhHKaaL7hQ6-nZnaha4CWVWEtIfYx4Vx53sxrChvlm25_EhXftu9Yo"
    )]
    identity_key_pair: NetworkIdentityKeypairPresentationDto,

    #[schema(example = json!([
        {
            "address": "/ip4/192.168.1.7/tcp/52310/p2p/12D3KooWSg4ox9udRcwrjo8ETg1gjB7g5wSSwjVMGKWJiqF9XjdB",
            "peer_id": "12D3KooWSg4ox9udRcwrjo8ETg1gjB7g5wSSwjVMGKWJiqF9XjdB",
        }
    ]))]
    network_addresses: Vec<NetworkAddressPresentationDto>,
}

impl From<GetNetworkSelfInfoUseCaseResponse> for GetNetworkSelfInfoHttpResponseBody {
    fn from(res: GetNetworkSelfInfoUseCaseResponse) -> Self {
        let identity_key_pair = res.identity_key_pair.into();
        let network_addresses = res
            .network_addresses
            .into_iter()
            .map(|addr| addr.into())
            .collect();
        Self {
            identity_key_pair,
            network_addresses,
        }
    }
}
