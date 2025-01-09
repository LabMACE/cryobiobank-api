use crate::config::Config;
use sea_orm::Database;

pub async fn check_external_services() {
    let config = Config::from_env();
    Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();
}
