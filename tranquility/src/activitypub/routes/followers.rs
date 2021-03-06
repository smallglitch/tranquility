use {
    super::CollectionQuery,
    crate::{
        activitypub::FollowActivity, consts::activitypub::ACTIVITIES_PER_PAGE, error::Error,
        format_uuid,
    },
    itertools::Itertools,
    tranquility_types::activitypub::{
        collection::Item, Actor, Collection, OUTBOX_FOLLOW_COLLECTIONS_PAGE_TYPE,
    },
    uuid::Uuid,
    warp::{Rejection, Reply},
};

pub async fn followers(user_id: Uuid, query: CollectionQuery) -> Result<impl Reply, Rejection> {
    let latest_follow_activities =
        crate::database::follow::followers(user_id, query.last_id, ACTIVITIES_PER_PAGE).await?;
    let last_id = latest_follow_activities
        .last()
        .map(|activity| format_uuid!(activity.id))
        .unwrap_or_default();

    let latest_followers = latest_follow_activities
        .into_iter()
        .filter_map(|activity| {
            let follow_activity: FollowActivity = serde_json::from_value(activity.data).ok()?;
            let follower_id = follow_activity.activity.id;

            Some(Item::Url(follower_id))
        })
        .collect_vec();

    let user_db = crate::database::actor::select::by_id(user_id).await?;
    let user: Actor = serde_json::from_value(user_db.actor).map_err(Error::from)?;

    let next = format!("{}?last_id={}", user.followers, last_id);

    let followers_collection = Collection {
        r#type: OUTBOX_FOLLOW_COLLECTIONS_PAGE_TYPE.into(),

        id: user.followers.clone(),
        part_of: user.followers,

        next,

        ordered_items: latest_followers,
        ..Collection::default()
    };

    Ok(warp::reply::json(&followers_collection))
}
