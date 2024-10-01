use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use cosmic::widget::svg;
use rust_embed::Embed;

pub(crate) struct IconCache {
    svg_cache: HashMap<&'static str, Cow<'static,[u8]>>,
    handle_cache: HashMap<&'static str, svg::Handle>,
}
#[derive(Embed)]
#[folder = "res/icons/"]
struct Icons;
impl IconCache {
    fn new() -> Self {
        let mut svg_cache = HashMap::new();
        let mut handle_cache = HashMap::new();

        macro_rules! bundle {
            ($name:expr) => {
                let path = concat!($name, ".svg");
                let data = Icons::get(path);
                if let Some(data) = data {
                    svg_cache.insert($name, data.data.clone());
                    handle_cache.insert($name, svg::Handle::from_memory(data.data));
                }
            };
        }
        bundle!("play");
        bundle!("pause");
        bundle!("stop");
        Self { svg_cache, handle_cache }
    }

    fn get_handle(&mut self, name: &'static str) -> svg::Handle {
        self.handle_cache
            .get(name)
            .unwrap()
            .clone()
    }
    fn get_svg(&mut self, name: &'static str) -> Cow<'static,[u8]> {
        self.svg_cache.get(name).unwrap().clone()
    }
}
static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();
pub(crate) fn get_icon_cache_handle(name: &'static str) -> svg::Handle {
    let mut icon_cache = ICON_CACHE
        .get_or_init(|| Mutex::new(IconCache::new()))
        .lock()
        .unwrap();
    icon_cache.get_handle(name)
}
pub(crate) fn get_icon_cache_svg(name: &'static str) -> Cow<'static,[u8]> {
    let mut icon_cache = ICON_CACHE
        .get_or_init(|| Mutex::new(IconCache::new()))
        .lock()
        .unwrap();
    icon_cache.get_svg(name)
}