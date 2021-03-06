use {
    crate::{
        config::ArcConfig, consts::cors::API_ALLOWED_METHODS, database::model::Actor, error::Error,
        util::construct_cors,
    },
    once_cell::sync::Lazy,
    serde::de::DeserializeOwned,
    tranquility_types::mastodon::App,
    warp::{reject::MissingHeader, Filter, Rejection, Reply},
};

static DEFAULT_APPLICATION: Lazy<App> = Lazy::new(|| App {
    name: "Web".into(),
    ..App::default()
});

pub fn urlencoded_or_json<T: DeserializeOwned + Send>(
) -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    let urlencoded_filter = warp::body::form();
    let json_filter = warp::body::json();

    urlencoded_filter.or(json_filter).unify()
}

pub fn authorization_optional() -> impl Filter<Extract = (Option<Actor>,), Error = Rejection> + Copy
{
    let or_none_fn = |error: Rejection| async move {
        if error.find::<MissingHeader>().is_some() {
            Ok((None,))
        } else {
            Err(error)
        }
    };

    authorization_required().map(Some).or_else(or_none_fn)
}

pub fn authorization_required() -> impl Filter<Extract = (Actor,), Error = Rejection> + Copy {
    let authorization_closure = |authorization_header: String| async move {
        let token = {
            let mut split_header = authorization_header.split_whitespace();

            let bearer_part = split_header.next().ok_or(Error::Unauthorized)?;
            if bearer_part.to_lowercase() != "bearer" {
                return Err::<_, Rejection>(Error::Unauthorized.into());
            }

            split_header.next().ok_or(Error::Unauthorized)?
        };

        let access_token = crate::database::oauth::token::select::by_token(token).await?;
        let actor = crate::database::actor::select::by_id(access_token.actor_id).await?;

        Ok(actor)
    };

    warp::header("authorization").and_then(authorization_closure)
}

pub fn routes(
    config: ArcConfig,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    // Enable CORS for all API endpoints
    // See: https://github.com/tootsuite/mastodon/blob/85324837ea1089c00fb4aefc31a7242847593b52/config/initializers/cors.rb
    let cors = construct_cors(API_ALLOWED_METHODS);

    let v1_prefix = warp::path!("api" / "v1" / ..);

    let accounts = accounts::routes(config.clone());
    let apps = apps::routes();
    let statuses = statuses::routes(config.clone());
    let instance = instance::routes(config);

    let v1_routes = accounts.or(apps).or(statuses).or(instance);

    v1_prefix.and(v1_routes).with(cors)
}

pub mod accounts;
pub mod apps;
pub mod convert;
pub mod instance;
pub mod statuses;
