use utoipa::{
    Modify, OpenApi,
    openapi::{
        self, LicenseBuilder,
        security::{ApiKey, ApiKeyValue, SecurityScheme},
    },
};

#[derive(OpenApi)]
#[openapi(modifiers(&OpenApiAuthAddon, &OpenApiPackageMetaInfoAddon))]
pub struct ApiDoc;

struct OpenApiAuthAddon;

impl Modify for OpenApiAuthAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        let api_key_value =
            ApiKeyValue::with_description("Authorization", "JWT authentication token");
        openapi.components = Some(
            utoipa::openapi::ComponentsBuilder::new()
                .security_scheme(
                    "api_jwt_token",
                    SecurityScheme::ApiKey(ApiKey::Header(api_key_value)),
                )
                // TODO: alternatively, passing auth as cookie is also possible, but there doesn't seem to be
                // a way to express this
                //
                // .security_scheme(
                //     "api_jwt_token",
                //     SecurityScheme::ApiKey(ApiKey::Cookie(api_key_value)),
                // )
                .build(),
        )
    }
}

struct OpenApiPackageMetaInfoAddon;

impl Modify for OpenApiPackageMetaInfoAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        openapi.info.title = String::from(env!("CARGO_PKG_NAME"));
        openapi.info.version = String::from(env!("CARGO_PKG_VERSION"));
        openapi.info.description = Some(String::from(env!("CARGO_PKG_DESCRIPTION")));
        openapi.info.license = Some(
            LicenseBuilder::new()
                .identifier(Some(env!("CARGO_PKG_LICENSE")))
                .name(env!("CARGO_PKG_LICENSE"))
                .build(),
        );
        openapi.info.contact = None;
    }
}
