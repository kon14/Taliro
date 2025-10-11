pub(crate) mod handlers;

use application::state::AppState;
use axum::routing::Router;
use handlers::dev::DevelopmentApiDoc;
use std::collections::BTreeMap;
use tower::Layer;
use tower_http::normalize_path::{NormalizePath, NormalizePathLayer};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Track: OpenAPI nested tags support.
// https://github.com/OAI/OpenAPI-Specification/issues/1367
// https://github.com/swagger-api/swagger-ui/issues/5969

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Taliro",
        description = "A simple UTXO/PoW-based blockchain written in Rust.",
    ),
    nest(
        (path = "/dev", api = DevelopmentApiDoc),
    ),
)]
struct ApiDoc;

impl ApiDoc {
    fn new(api_base_url: &str, use_master_key: bool) -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();
        doc.servers = Some(vec![utoipa::openapi::Server::new(api_base_url)]);

        if use_master_key {
            let mut security_schemes = BTreeMap::new();
            security_schemes.insert(
                "bearerAuth".to_string(),
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
            let mut components = doc.components.unwrap_or_default();
            components.security_schemes = security_schemes;
            doc.components = Some(components);
        }

        doc
    }
}

pub(crate) fn build_router(app_state: AppState, api_base_url: &str) -> NormalizePath<Router> {
    let use_master_key = app_state.master_key_authenticator.is_enabled();
    let router = Router::new()
        .merge(setup_swagger_ui(api_base_url, use_master_key))
        .merge(handlers::dev::declare_routes("/dev"))
        .with_state(app_state);

    // Fix trailing slash endpoints
    NormalizePathLayer::trim_trailing_slash().layer(router)
}

fn setup_swagger_ui(api_base_url: &str, use_master_key: bool) -> SwaggerUi {
    // NormalizeLayer breaks Swagger UI
    const SWAGGER_UI_PATH: &str = "/swagger"; // /swagger/index.html
    const SWAGGER_API_DOC_PATH: &str = "/swagger.json";

    let config = utoipa_swagger_ui::Config::new([SWAGGER_API_DOC_PATH])
        .try_it_out_enabled(true)
        .persist_authorization(use_master_key);

    SwaggerUi::new(SWAGGER_UI_PATH).config(config).url(
        SWAGGER_API_DOC_PATH,
        ApiDoc::new(api_base_url, use_master_key),
    )
}
