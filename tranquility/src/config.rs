use {
    serde::Deserialize,
    std::{convert::Infallible, sync::Arc},
    tokio::{
        fs::File,
        io::{AsyncReadExt, BufReader},
    },
    warp::Filter,
};

#[allow(clippy::module_name_repetitions)]
pub type ArcConfig = Arc<Configuration>;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationInstance {
    pub closed_registrations: bool,
    pub domain: String,

    pub description: String,

    pub character_limit: usize,
    pub upload_limit: usize,

    pub moderators: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationRatelimit {
    pub active: bool,

    pub authentication_quota: u32,
    pub registration_quota: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationServer {
    pub port: u16,

    pub database_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationTls {
    pub serve_tls_directly: bool,

    pub certificate: String,
    pub secret_key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub instance: ConfigurationInstance,
    pub ratelimit: ConfigurationRatelimit,
    pub server: ConfigurationServer,
    pub tls: ConfigurationTls,
}

pub async fn load(config_path: String) -> Configuration {
    let config_file = File::open(config_path)
        .await
        .expect("Couldn't open configuration file");
    let mut config_file = BufReader::new(config_file);

    let mut data = Vec::new();
    config_file
        .read_to_end(&mut data)
        .await
        .expect("Couldn't read configuration file");

    toml::from_slice(data.as_slice()).expect("Invalid TOML")
}

pub fn filter(
    config: ArcConfig,
) -> impl Filter<Extract = (ArcConfig,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&config))
}
