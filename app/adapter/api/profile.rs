use application::{
    auth::{
        command::sign_out::{SignOutCommand, SignOutCommandHandler},
        service::auth_service::AuthService,
    },
    organization::{
        command::update_user_self_password::{
            UpdateUserSelfPasswordCommand, UpdateUserSelfPasswordCommandHandler,
        },
        query::retrieve_user::{RetrieveUserQuery, RetrieveUserQueryHandler},
    },
    shared::{command_handler::CommandHandler, query_handler::QueryHandler as _},
};

use axum::Json;
use domain::auth::value_object::menu::MenuTree;
use i18n::LanguageIdentifier;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState,
    api::profile::response::TranslatedMenuTree,
    i18n::LOCALES,
    shared::{
        extractor::{accept_language::AcceptLanguage, inject::Inject, valid_user::ValidUser},
        response::{JsonResponse, JsonResponseEmpty, JsonResponseType},
    },
};

#[utoipa::path(
    post,
    path = "/sign_out",
    summary = "Sign out",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn sign_out(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<SignOutCommandHandler>,
) -> JsonResponseType<()> {
    let command = SignOutCommand::builder().id(id).build();
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    get,
    path = "/",
    summary = "Current user",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponse<response::CurrentResponse>))
    )
)]
#[tracing::instrument]
async fn current(
    language: AcceptLanguage,
    ValidUser(id): ValidUser,
    Inject(service): Inject<AuthService>,
    Inject(query_handler): Inject<RetrieveUserQueryHandler>,
) -> JsonResponseType<response::CurrentResponse> {
    let (mut user, pages, permissions) = tokio::try_join!(
        query_handler.query(RetrieveUserQuery::builder().id(id.clone()).build()),
        async { Ok(service.get_available_pages(&id).await) },
        async { Ok(service.get_available_permissions(&id).await) }
    )?;
    service
        .replenish_user_portrait(std::slice::from_mut(&mut user))
        .await;
    JsonResponse::ok(response::CurrentResponse {
        user,
        pages: tranlate_menus(pages.to_vec(), language.identifier()),
        permissions,
    })
}

fn tranlate_menus(
    menus: Vec<MenuTree>,
    lang_id: &LanguageIdentifier,
) -> Vec<response::TranslatedMenuTree> {
    let items = menus
        .into_iter()
        .map(Into::into)
        .collect::<Vec<TranslatedMenuTree>>();
    tranlate_menus_inner(&items, lang_id)
}

fn tranlate_menus_inner(
    menus: &[TranslatedMenuTree],
    lang_id: &LanguageIdentifier,
) -> Vec<response::TranslatedMenuTree> {
    let mut menus = menus.to_vec();
    for menu in menus.iter_mut() {
        if let Some(label) = &menu.label {
            let query = i18n::Query::new(label);
            menu.label = LOCALES
                .query(lang_id, &query)
                .map(|message| message.value)
                .ok();
        }
        if let Some(children) = &menu.children {
            menu.children = Some(tranlate_menus_inner(children, lang_id));
        }
    }
    menus
}

#[utoipa::path(
    post,
    path = "/password",
    summary = "Change self password",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument(skip(request))]
async fn password(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<UpdateUserSelfPasswordCommandHandler>,
    Json(request): Json<request::UpdatePasswordRequest>,
) -> JsonResponseType<()> {
    let command = UpdateUserSelfPasswordCommand::builder()
        .id(id)
        .password(request.password)
        .new_password(request.new_password)
        .confirm_new_password(request.confirm_new_password)
        .build();
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

mod request {
    use serde::Deserialize;
    use utoipa::ToSchema;

    #[derive(Deserialize, ToSchema)]
    pub struct UpdatePasswordRequest {
        pub password: String,
        pub new_password: String,
        pub confirm_new_password: String,
    }
}
mod response {
    use application::organization::dto::user::UserDto;
    use domain::auth::value_object::{
        menu::{Menu, MenuTree},
        permission::Permission,
    };
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct CurrentResponse {
        pub user: UserDto,
        pub pages: Vec<TranslatedMenuTree>,
        pub permissions: Vec<&'static Permission>,
    }

    #[derive(Debug, Clone, Serialize, ToSchema)]
    pub struct TranslatedMenuTree {
        pub key: Menu,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub link: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub redirect: Option<String>,
        #[serde(rename = "schemaApi", skip_serializing_if = "Option::is_none")]
        pub schema_api: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(no_recursion)]
        pub children: Option<Vec<TranslatedMenuTree>>,
        #[serde(skip_serializing_if = "is_true")]
        pub visible: bool,
    }

    fn is_true(b: &bool) -> bool {
        *b
    }

    impl From<MenuTree> for TranslatedMenuTree {
        fn from(value: MenuTree) -> Self {
            Self {
                key: value.key,
                label: value.label.map(ToString::to_string),
                icon: value.icon.map(ToString::to_string),
                url: value.url.map(ToString::to_string),
                link: value.link.map(ToString::to_string),
                redirect: value.redirect.map(ToString::to_string),
                schema_api: value.schema_api.map(ToString::to_string),
                children: value
                    .children
                    .map(|d| d.into_iter().map(Into::into).collect()),
                visible: value.visible,
            }
        }
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(current))
        .routes(routes!(sign_out))
        .routes(routes!(password))
}
