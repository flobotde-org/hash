//! The Axum webserver for accessing the Graph API operations.
//!
//! Handler methods are grouped by routes that make up the REST API.

mod api_resource;
mod data_type;
mod entity_type;
mod link_type;
mod property_type;

use axum::{routing::get, Extension, Json, Router};
use utoipa::{openapi, Modify, OpenApi};

use self::api_resource::RoutedResource;
use crate::store::Store;

fn api_resources<T: Store>() -> Vec<Router> {
    vec![
        data_type::DataTypeResource::routes::<T>(),
        property_type::PropertyTypeResource::routes::<T>(),
        link_type::LinkTypeResource::routes::<T>(),
        entity_type::EntityTypeResource::routes::<T>(),
    ]
}

fn api_documentation() -> Vec<openapi::OpenApi> {
    vec![
        data_type::DataTypeResource::documentation(),
        property_type::PropertyTypeResource::documentation(),
        link_type::LinkTypeResource::documentation(),
        entity_type::EntityTypeResource::documentation(),
    ]
}

pub fn rest_api_router<T: Store>(store: T) -> Router {
    // All api resources are merged together into a super-router.
    let merged_routes = api_resources::<T>()
        .into_iter()
        .fold(Router::new(), axum::Router::merge);

    // OpenAPI documentation is also generated by merging resources
    let open_api_doc = OpenApiDocumentation::openapi();

    // super-router can then be used as any other router.
    merged_routes
        // Make sure extensions are added at the end so they are made available to merged routers.
        .layer(Extension(store))
        .route(
            "/api-doc/openapi.json",
            get({
                let doc = open_api_doc;
                move || async { Json(doc) }
            }),
        )
}

#[derive(OpenApi)]
#[openapi(
        tags(
            (name = "Graph", description = "HASH Graph API")
        ),
        modifiers(&MergeAddon)
    )]
struct OpenApiDocumentation;

struct MergeAddon;

impl Modify for MergeAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let api_documentation = api_documentation();

        let api_components = api_documentation
            .iter()
            .cloned()
            .filter_map(|api_docs| {
                api_docs
                    .components
                    .map(|components| components.schemas.into_iter())
            })
            .flatten();

        let mut components = openapi.components.take().unwrap_or_default();
        components.schemas.extend(api_components);
        openapi.components = Some(components);

        let mut tags = openapi.tags.take().unwrap_or_default();
        tags.extend(
            api_documentation
                .iter()
                .cloned()
                .filter_map(|api_docs| api_docs.tags)
                .flatten(),
        );
        openapi.tags = Some(tags);

        openapi.paths.paths.extend(
            api_documentation
                .iter()
                .cloned()
                .flat_map(|api_docs| api_docs.paths.paths.into_iter()),
        );
    }
}
