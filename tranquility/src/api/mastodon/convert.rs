use {
    crate::{
        activitypub::ActivityObject,
        database::model::{Actor as DBActor, OAuthApplication, Object as DBObject},
        error::Error,
    },
    async_trait::async_trait,
    serde::Serialize,
    tranquility_types::{
        activitypub::{Activity, Actor, Object},
        mastodon::{Account, App, Source, Status},
    },
    url::Url,
    warp::Rejection,
};

#[async_trait]
pub trait IntoMastodon<ApiEntity>: Clone + Send + Sync
where
    ApiEntity: Serialize + 'static,
{
    type Error: Into<Rejection>;

    async fn into_mastodon(self) -> Result<ApiEntity, Self::Error>;

    async fn into_mastodon_cloned(&self) -> Result<ApiEntity, Self::Error> {
        self.clone().into_mastodon().await
    }
}

#[async_trait]
impl IntoMastodon<Status> for Activity {
    type Error = Error;

    async fn into_mastodon(self) -> Result<Status, Self::Error> {
        let object =
            crate::activitypub::fetcher::fetch_object(self.object.as_url().unwrap()).await?;

        object.into_mastodon().await
    }
}

#[async_trait]
impl IntoMastodon<Status> for ActivityObject {
    type Error = Error;

    async fn into_mastodon(self) -> Result<Status, Self::Error> {
        match self {
            ActivityObject::Activity(activity) => activity.into_mastodon(),
            ActivityObject::Object(object) => object.into_mastodon(),
        }
        .await
    }
}

#[async_trait]
impl IntoMastodon<Account> for DBActor {
    type Error = Error;

    async fn into_mastodon(self) -> Result<Account, Self::Error> {
        let actor: Actor = serde_json::from_value(self.actor)?;

        let id = self.id.to_simple().to_string();
        let username = actor.username;
        let url = actor.id;
        let acct = if self.remote {
            let parsed_url = Url::parse(&url)?;

            format!(
                "{}@{}",
                username,
                parsed_url.host_str().ok_or(Error::MalformedUrl)?
            )
        } else {
            username.clone()
        };
        let display_name = actor.name;
        let avatar = actor
            .icon
            .map(|attachment| attachment.url)
            .unwrap_or_default();
        let header = actor
            .image
            .map(|attachment| attachment.url)
            .unwrap_or_default();

        Ok(Account {
            id,
            username,
            acct,
            display_name,

            avatar_static: avatar.clone(),
            avatar,

            header_static: header.clone(),
            header,
            ..Account::default()
        })
    }
}

#[async_trait]
impl IntoMastodon<Source> for DBActor {
    type Error = Error;

    async fn into_mastodon(self) -> Result<Source, Self::Error> {
        let actor: Actor = serde_json::from_value(self.actor)?;

        let source = Source {
            privacy: "public".into(),
            language: "en".into(),

            note: actor.summary,

            ..Source::default()
        };

        Ok(source)
    }
}

#[async_trait]
impl IntoMastodon<Status> for DBObject {
    type Error = Error;

    async fn into_mastodon(self) -> Result<Status, Self::Error> {
        let activity_or_object: ActivityObject = serde_json::from_value(self.data)?;

        activity_or_object.into_mastodon().await
    }
}

#[async_trait]
impl IntoMastodon<App> for OAuthApplication {
    type Error = Error;

    async fn into_mastodon(self) -> Result<App, Self::Error> {
        let id = self.id.to_simple().to_string();
        let client_id = self.client_id.to_simple().to_string();
        let website = if self.website.is_empty() {
            None
        } else {
            Some(self.website)
        };

        let app = App {
            id,
            name: self.client_name,
            client_id,
            client_secret: self.client_secret,
            redirect_uri: self.redirect_uris,
            website,
            vapid_key: None,
        };

        Ok(app)
    }
}

#[async_trait]
impl IntoMastodon<Status> for Object {
    type Error = Error;

    async fn into_mastodon(self) -> Result<Status, Self::Error> {
        let db_object = crate::database::object::select::by_url(self.id.as_str()).await?;
        let (_actor, db_actor) =
            crate::activitypub::fetcher::fetch_actor(self.attributed_to.as_str()).await?;

        let id = db_object.id.to_simple().to_string();
        let application = super::DEFAULT_APPLICATION.clone();
        let account = db_actor.into_mastodon().await?;

        let status = Status {
            id,
            created_at: self.published,

            sensitive: self.sensitive,
            spoiler_text: self.summary,
            visibility: "public".into(),

            uri: self.id.clone(),
            url: self.id,

            content: self.content,

            application,
            account,

            ..Status::default()
        };

        Ok(status)
    }
}
