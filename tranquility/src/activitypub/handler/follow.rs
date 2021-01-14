use {
    crate::{
        activitypub::{self, deliverer, fetcher, FollowActivity},
        error::Error,
    },
    tranquility_types::activitypub::{activity::ObjectField, Activity},
    warp::http::StatusCode,
};

pub async fn handle(mut activity: Activity) -> Result<StatusCode, Error> {
    let actor_url = match activity.object {
        ObjectField::Actor(ref actor) => actor.id.as_str(),
        ObjectField::Url(ref url) => url.as_str(),
        ObjectField::Object(_) => return Err(Error::UnknownActivity),
    };

    // Fetch the actor (just in case)
    let (actor, actor_db) = fetcher::fetch_actor(actor_url).await?;

    // Normalize the activity
    if let ObjectField::Actor(actor) = activity.object {
        activity.object = ObjectField::Url(actor.id);
    }

    // Automatically approve the follow (this will be choice based at some point)
    let follow_activity = FollowActivity {
        activity,
        approved: true,
    };
    let activity = serde_json::to_value(&follow_activity)?;

    crate::database::object::insert(actor_db.id, &follow_activity.activity.id, activity).await?;

    let followed_url = follow_activity.activity.object.as_url().unwrap();
    let followed_actor = crate::database::actor::select::by_url(followed_url).await?;

    // Send out an accept activity if the followed actor is local
    if follow_activity.approved {
        let (_accept_activity_id, accept_activity) = activitypub::instantiate::activity(
            "Accept",
            followed_url,
            follow_activity.activity.id,
            vec![actor.id],
            Vec::new(),
        );
        let accept_activity_value = serde_json::to_value(&accept_activity)?;

        crate::database::object::insert(
            followed_actor.id,
            &accept_activity.id,
            accept_activity_value,
        )
        .await?;

        deliverer::deliver(accept_activity).await?;
    }

    Ok(StatusCode::CREATED)
}
