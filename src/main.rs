extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate gdcf;
extern crate gdcf_dbcache;
extern crate gdrs;

use chrono::Duration;
use futures::{Async, Future};
use gdcf::{ConsistentCacheManager, Gdcf};
use gdcf::api::request::{LevelRequest, LevelsRequest, Request};
use gdcf::cache::Cache;
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;


fn main() {
    env_logger::init();

    // Rust built-in await/async WHEN

    let config = DatabaseCacheConfig::postgres_config("postgres://gdcf:gdcf@localhost/gdcf");
    let cache = DatabaseCache::new(config);

    cache.initialize().expect("Error initializing cache");

    let client = BoomlingsClient::new();
    let gdcf = ConsistentCacheManager::new(client, cache);

    for id in vec![38515466u64, 47620786, 47998429, 47849218, 47339027] {
        println!(
            "{:?}",
            gdcf.level(LevelRequest::new(id))
                .map(|l| l.password)
        );
    }

    gdcf.levels(LevelsRequest::new().search("Starfire".into()));
}

pub fn until_all_done() -> impl Future<Item=(), Error=()> {
    Thing {}
}

struct Thing;

impl Future for Thing {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<()>, ()> {
        Ok(Async::NotReady)
    }
}