use {
    std::{future::Future, time::Duration},
    tokio::time,
};

const DELETE_INTERVAL: Duration = Duration::from_secs(60);

// Keeping this for future use
#[allow(dead_code)]
fn bulk_spawn(futures: Vec<impl Future<Output = ()> + Send + Sync + 'static>) {
    for future in futures {
        tokio::spawn(future);
    }
}

async fn delete_expired_authorization_codes() {
    let mut query_interval = time::interval(DELETE_INTERVAL);

    loop {
        match crate::database::oauth::authorization::delete::expired().await {
            Ok(_) => (),
            Err(err) => warn!("Couldn't delete expired tokens: {}", err),
        }

        query_interval.tick().await;
    }
}

pub fn start() {
    tokio::spawn(delete_expired_authorization_codes());
}
