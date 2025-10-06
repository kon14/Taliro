use common::params::PaginationParams;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[schema(title = "PaginationParams")]
pub(crate) struct PaginationParamsPresentationDto {
    skip: usize,
    limit: usize,
}

impl From<PaginationParamsPresentationDto> for PaginationParams {
    fn from(params: PaginationParamsPresentationDto) -> Self {
        Self {
            skip: params.skip,
            limit: params.limit,
        }
    }
}
