use std::{
    cell::RefCell,
    collections::{
        btree_map::Entry,
        BTreeMap,
    },
    sync::Arc,
    time::Instant,
};

struct CacheEntry<T> {
    value: Arc<T>,
    last_use: Instant,
    flag_used: bool,
}

impl<T> CacheEntry<T> {
    pub fn create(value: T) -> Self {
        Self {
            value: Arc::new(value),
            last_use: Instant::now(),
            flag_used: false,
        }
    }

    pub fn flag_use(&mut self) {
        self.flag_used = true;
    }

    /// Commits the used flag.
    /// Returns the seconds since last use.
    pub fn commit_use(&mut self) -> u64 {
        if self.flag_used {
            self.flag_used = false;
            self.last_use = Instant::now();
            0
        } else {
            self.last_use.elapsed().as_secs()
        }
    }
}

pub struct EntryCache<K, V> {
    loader: Box<dyn Fn(&K) -> anyhow::Result<V>>,
    cache: RefCell<BTreeMap<K, CacheEntry<V>>>,
}

impl<K: Ord, V> EntryCache<K, V> {
    pub fn new(loader: impl Fn(&K) -> anyhow::Result<V> + 'static) -> Self {
        Self {
            loader: Box::new(loader),
            cache: Default::default(),
        }
    }

    pub fn lookup(&self, key: K) -> anyhow::Result<Arc<V>> {
        let mut cache = self.cache.borrow_mut();
        let entry = match cache.entry(key) {
            Entry::Occupied(value) => value.into_mut(),
            Entry::Vacant(entry) => {
                let value = (self.loader)(entry.key())?;
                entry.insert(CacheEntry::create(value))
            }
        };
        entry.flag_use();

        Ok(entry.value.clone())
    }

    pub fn cleanup(&self) {}
}
