use std::{
    collections::HashSet,
    hash::Hash,
    sync::{Arc, Mutex},
};

use image::DynamicImage;
use schnellru::{Limiter, LruMap};

use crate::{events::Event, traits::HeapSize, ui::common::Image};

type F<K> = dyn Send + Sync + Fn(&K) -> Option<DynamicImage>;

pub struct ImageCache<K> {
    state: Arc<Mutex<State<K>>>,
    get_image: Arc<F<K>>,
}

struct State<K> {
    map: LruMap<K, Arc<Image>, ByHeapSize>,
    fetching: HashSet<K>,
    missing: HashSet<K>,
}

impl<K> Default for State<K>
where
    K: Hash + PartialEq,
{
    fn default() -> Self {
        let limiter = ByHeapSize::new(1024 * 1024 * 256);
        Self {
            map: LruMap::with_hasher(limiter, Default::default()),
            fetching: Default::default(),
            missing: Default::default(),
        }
    }
}

impl<K> ImageCache<K>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
{
    pub fn new(get_image: impl Fn(&K) -> Option<DynamicImage> + Send + Sync + 'static) -> Self {
        Self {
            state: Default::default(),
            get_image: Arc::new(get_image),
        }
    }

    pub fn get(&self, key: &K) -> Option<Arc<Image>> {
        {
            let mut lock = self.state.lock().unwrap();

            if lock.fetching.contains(key) || lock.missing.contains(key) {
                return None;
            }

            if let Some(img) = lock.map.get(key) {
                return Some(Arc::clone(img));
            }

            lock.fetching.insert(key.clone());
        }

        let key = key.clone();
        let state = Arc::clone(&self.state);
        let get_image = Arc::clone(&self.get_image);
        tokio::task::spawn_blocking(move || {
            if let Some(img) = (*get_image)(&key) {
                let img = Image::new(img);
                {
                    let mut lock = state.lock().unwrap();
                    lock.fetching.remove(&key);
                    lock.map.insert(key, Arc::new(img));
                }
                Event::Render.emit();
            } else {
                let mut lock = state.lock().unwrap();
                lock.fetching.remove(&key);
                lock.missing.insert(key);
            }
        });

        None
    }
}

struct ByHeapSize {
    bytes: usize,
    max_bytes: usize,
}

impl ByHeapSize {
    fn new(max_bytes: usize) -> Self {
        Self {
            bytes: 0,
            max_bytes,
        }
    }
}

impl<K, V> Limiter<K, Arc<V>> for ByHeapSize
where
    V: HeapSize,
{
    type KeyToInsert<'a> = K;
    type LinkType = u32;

    #[inline]
    fn is_over_the_limit(&self, _length: usize) -> bool {
        self.bytes > self.max_bytes
    }

    #[inline]
    fn on_insert(&mut self, _length: usize, key: K, value: Arc<V>) -> Option<(K, Arc<V>)> {
        let byte_size = value.byte_size();
        if byte_size <= self.max_bytes {
            self.bytes += byte_size;
            Some((key, value))
        } else {
            None
        }
    }

    #[inline]
    fn on_replace(
        &mut self,
        _length: usize,
        _old_key: &mut K,
        _new_key: K,
        old_value: &mut Arc<V>,
        new_value: &mut Arc<V>,
    ) -> bool {
        self.bytes -= old_value.byte_size();
        self.bytes += new_value.byte_size();
        true
    }

    #[inline]
    fn on_removed(&mut self, _key: &mut K, value: &mut Arc<V>) {
        self.bytes -= value.byte_size();
    }

    #[inline]
    fn on_cleared(&mut self) {
        self.bytes = 0;
    }

    #[inline]
    fn on_grow(&mut self, _new_memory_usage: usize) -> bool {
        true
    }
}
