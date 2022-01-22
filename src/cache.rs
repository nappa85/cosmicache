use std::{time::Duration, sync::Arc};

use arc_swap::ArcSwap;

use mysql_async::prelude::Queryable;

use once_cell::sync::Lazy;

use tracing::{error, info};

use crate::{Instance, Device};

pub static INSTANCES: Lazy<ArcSwap<Vec<Instance>>> = Lazy::new(Default::default);
pub static DEVICES: Lazy<ArcSwap<Vec<Device>>> = Lazy::new(Default::default);

async fn _cache() -> Result<(), ()> {
    let mut conn = crate::db::MYSQL.get_conn().await.map_err(|e| error!("MySQL connect error: {}", e))?;

    let res = conn.query_iter("SELECT `name`, `type`, `data` FROM `instance`").await.map_err(|e| error!("MySQL retrieve instances error: {}", e))?;
    let instances = res.map_and_drop(|mut row| {
        Instance {
            name: row.take_opt("name").expect("instance.name").expect("instance.name"),
            _type: row.take_opt("type").expect("instance.type").expect("instance.type"),
            data: row.take_opt("data").expect("instance.data").expect("instance.data"),
        }
    }).await.map_err(|e| error!("MySQL collect instances error: {}", e))?;
    let n_instances = instances.len();
    INSTANCES.swap(Arc::new(instances));

    let res = conn.query_iter("SELECT `uuid`, `instance_name`, `last_seen` FROM `device`").await.map_err(|e| error!("MySQL retrieve instances error: {}", e))?;
    let devices = res.map_and_drop(|mut row| {
        Device {
            uuid: row.take_opt("uuid").expect("device.uuid").expect("device.uuid"),
            instance_name: row.take_opt("instance_name").expect("device.instance_name").expect("device.instance_name"),
            last_seen: row.take_opt("last_seen").expect("device.last_seen").expect("device.last_seen"),
        }
    }).await.map_err(|e| error!("MySQL collect instances error: {}", e))?;
    let n_devices = devices.len();
    DEVICES.swap(Arc::new(devices));

    info!("Cache loaded {} instances and {} devices", n_instances, n_devices);

    Ok(())
}

pub async fn cache() {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        _cache().await.ok();
    }
}
