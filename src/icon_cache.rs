use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use cosmic::widget::svg;

pub(crate) struct IconCache{
    cache: HashMap<&'static str, svg::Handle>,
}

impl IconCache {
    fn new() -> Self{
        let mut cache = HashMap::new();
        macro_rules! bundle {
            ($name:expr) => {
                let data: &'static [u8] = include_bytes!(concat!("../res/icons/", $name, ".svg"));
                cache.insert(
                    $name,
                    svg::Handle::from_memory(data),
                );
            };
        }
        bundle!("play");
        bundle!("pause");
        Self { cache }
    }

    fn get(&mut self, name: &'static str) -> svg::Handle {
        self.cache
            .get(name)
            .unwrap()
            .clone()
    }



}
static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();
pub(crate) fn get_icon_cache_handle(name: &'static str) -> svg::Handle {
    let mut icon_cache = ICON_CACHE
        .get_or_init(|| Mutex::new(IconCache::new()))
        .lock()
        .unwrap();
    icon_cache.get(name)
}