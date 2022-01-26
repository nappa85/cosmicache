use std::convert::Infallible;

use serde::Serialize;

use warp::Filter;

mod db;
mod cache;

#[derive(Debug, Serialize)]
pub struct Instance {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct Device {
    pub uuid: String,
    pub instance_name: Option<String>,
    pub last_seen: i64,
}

async fn get_all_instances() -> Result<impl warp::Reply, Infallible> {
    get_instances(String::new()).await
}

async fn get_instances(instances: String) -> Result<impl warp::Reply, Infallible> {
    let instances = instances.replace("%20", " ");
    let instances = if instances.is_empty() { vec![] } else { instances.split(',').collect::<Vec<_>>() };
    let data = cache::INSTANCES.load();
    let temp = data.iter()
        .filter_map(|o| (instances.is_empty() || instances.contains(&o.name.as_str())).then(|| o))
        .collect::<Vec<_>>();
    Ok(warp::reply::json(&temp))
}

async fn get_all_devices() -> Result<impl warp::Reply, Infallible> {
    get_devices(String::new()).await
}

async fn get_devices(instances: String) -> Result<impl warp::Reply, Infallible> {
    let instances = instances.replace("%20", " ");
    let instances = if instances.is_empty() { vec![] } else { instances.split(',').collect::<Vec<_>>() };
    let data = cache::DEVICES.load();
    let temp = data.iter()
        .filter_map(|o| (instances.is_empty() || instances.contains(&o.instance_name.as_deref().unwrap_or_default())).then(|| o))
        .collect::<Vec<_>>();
    Ok(warp::reply::json(&temp))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tokio::spawn(cache::cache());

    let routes = warp::get()
        .and(warp::path!("instances" / String).and_then(get_instances))
        .or(warp::path!("instances").and_then(get_all_instances))
        .or(warp::path!("devices" / String).and_then(get_devices))
        .or(warp::path!("devices").and_then(get_all_devices));

    warp::serve(routes)
        .run(([0, 0, 0, 0], 9991))
        .await
}
