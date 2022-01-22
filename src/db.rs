use std::env;

use mysql_async::Pool;

use once_cell::sync::Lazy;

pub static MYSQL: Lazy<Pool> = Lazy::new(|| Pool::new(env::var("DATABASE_URL").unwrap().as_str()));
