use std::{
    self,
    any::{
        self,
        Any,
        TypeId,
    },
    cell::{
        Ref,
        RefCell,
        RefMut,
    },
    collections::{
        hash_map::Entry,
        HashMap,
    },
    hash::{
        DefaultHasher,
        Hash,
        Hasher,
    },
    ops::DerefMut,
    time::{
        Duration,
        Instant,
    },
};

use anyhow::{
    anyhow,
    Context,
};

pub enum StateCacheType {
    /// The state will be cached and never removed
    Persistent,

    /// The cache entry will be invalidated if not accessed within
    /// the target durection.
    Timed(Duration),

    /// The state will be removed as soon it get's invalidated.
    /// The update method will only be called once uppon creation.
    Volatile,
}

pub trait State: Any + Sized + Send {
    type Parameter: Hash + PartialEq;

    /// Create a new instance of this state.
    /// Note: update will be called after creation automatically.
    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        anyhow::bail!("state must be manually set")
    }

    /// Return how the state should be cached
    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }

    /// Update the state
    fn update(&mut self, _states: &StateRegistry) -> anyhow::Result<()> {
        Ok(())
    }
}

fn value_update_proxy<T: State>(
    value: &mut Box<dyn Any + Send>,
    states: &StateRegistry,
) -> anyhow::Result<()> {
    let value = value.downcast_mut::<T>().expect("to be of type T");
    value.update(states)
}

struct InternalState {
    value: Box<dyn Any + Send>,
    value_update: fn(&mut Box<dyn Any + Send>, states: &StateRegistry) -> anyhow::Result<()>,

    cache_key: (TypeId, u64),
    cache_type: StateCacheType,

    dirty: bool,
    last_access: Instant,
}

struct StateAllocator {
    index_lookup: HashMap<(TypeId, u64), usize>,
    free_list: Vec<usize>,
}

impl StateAllocator {
    pub fn new(capacity: usize) -> Self {
        let mut free_list = Vec::with_capacity(capacity);
        for index in (0..capacity).rev() {
            free_list.push(index);
        }

        Self {
            index_lookup: Default::default(),
            free_list,
        }
    }

    fn calculate_state_index<T: State>(
        &mut self,
        params: &T::Parameter,
        create_if_not_exists: bool,
    ) -> Option<((TypeId, u64), usize)> {
        let mut hasher = DefaultHasher::new();
        params.hash(&mut hasher);
        let params_hash = hasher.finish();

        let cache_key = (TypeId::of::<T>(), params_hash);
        let index = match self.index_lookup.entry(cache_key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                if !create_if_not_exists {
                    /* Do not create the target entry */
                    return None;
                }

                let index = self.free_list.pop()?;
                *entry.insert(index)
            }
        };

        Some((cache_key, index))
    }

    fn free_entry(&mut self, cache_key: &(TypeId, u64)) {
        let index = match self.index_lookup.remove(cache_key) {
            Some(index) => index,
            None => return,
        };
        self.free_list.push(index);
    }
}

fn transpose_ref_opt<T>(x: Ref<'_, Option<T>>) -> Option<Ref<'_, T>> {
    if x.is_none() {
        None
    } else {
        Some(Ref::map(x, |x| x.as_ref().unwrap()))
    }
}

fn transpose_ref_mut_opt<T>(x: RefMut<'_, Option<T>>) -> Option<RefMut<'_, T>> {
    if x.is_none() {
        None
    } else {
        Some(RefMut::map(x, |x| x.as_mut().unwrap()))
    }
}

pub struct StateRegistry {
    allocator: RefCell<StateAllocator>,
    states: Vec<RefCell<Option<InternalState>>>,
}

impl StateRegistry {
    pub fn new(capacity: usize) -> Self {
        let mut states = Vec::with_capacity(capacity);
        states.resize_with(capacity, Default::default);
        Self {
            allocator: RefCell::new(StateAllocator::new(capacity)),
            states,
        }
    }

    pub fn invalidate_states(&mut self) {
        /* As we're mutable there should be no more references to the underlying state */
        let mut allocator = self.allocator.borrow_mut();

        let now = Instant::now();
        for state in self.states.iter_mut() {
            let mut state_ref = state.borrow_mut();
            let state = if let Some(state) = state_ref.deref_mut() {
                state
            } else {
                continue;
            };

            if !state.dirty {
                /* State has been accessed. */
                state.last_access = now;
                state.dirty = true;
            }

            let state_expired = match state.cache_type {
                StateCacheType::Persistent => false,
                StateCacheType::Volatile => true,
                StateCacheType::Timed(timeout) => state.last_access.elapsed() > timeout,
            };
            if state_expired {
                allocator.free_entry(&state.cache_key);
                *state_ref = None;
            }
        }
    }

    /// Preset a specific state
    pub fn set<T: State>(&mut self, value: T, params: T::Parameter) -> anyhow::Result<()> {
        let (cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T>(&params, true)
            .context("state capacity exceeded")?;

        let mut state_ref = self.states[index].borrow_mut();
        *state_ref = Some(InternalState {
            value: Box::new(value),
            value_update: value_update_proxy::<T>,

            cache_key,
            cache_type: T::cache_type(),

            dirty: false,
            last_access: Instant::now(),
        });
        Ok(())
    }

    pub fn get<T: State>(&self, params: T::Parameter) -> Option<Ref<'_, T>> {
        let (_cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T>(&params, false)?;

        let value = self.states[index]
            .try_borrow()
            .ok()
            .map(transpose_ref_opt)
            .flatten()?;

        let value = Ref::map(value, |value| {
            value.value.downcast_ref::<T>().expect("to be type T")
        });

        Some(value)
    }

    pub fn get_mut<T: State>(&self, params: T::Parameter) -> Option<RefMut<'_, T>> {
        let (_cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T>(&params, false)?;

        let value = self.states[index]
            .try_borrow_mut()
            .ok()
            .map(transpose_ref_mut_opt)
            .flatten()?;

        let value = RefMut::map(value, |value| {
            value.value.downcast_mut::<T>().expect("to be type T")
        });

        Some(value)
    }

    fn initialize_value<T: State>(
        &self,
        cache_key: (TypeId, u64),
        value: &mut RefMut<'_, Option<InternalState>>,
        params: T::Parameter,
    ) -> anyhow::Result<()> {
        let value = match value.as_mut() {
            Some(value) => value,
            None => {
                /* create a new value */
                let state = Box::new(
                    T::create(self, params)
                        .with_context(|| format!("create {}", any::type_name::<T>()))?,
                );
                **value = Some(InternalState {
                    value: state,
                    value_update: value_update_proxy::<T>,

                    cache_key,
                    cache_type: T::cache_type(),

                    dirty: true,
                    last_access: Instant::now(),
                });

                value.as_mut().unwrap()
            }
        };

        if value.dirty {
            (value.value_update)(&mut value.value, self)
                .with_context(|| format!("update {}", any::type_name::<T>()))?;
            value.dirty = false;
        }

        Ok(())
    }

    pub fn resolve_mut<T: State>(&self, params: T::Parameter) -> anyhow::Result<RefMut<'_, T>> {
        let (cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T>(&params, true)
            .context("state capacity exceeded")?;

        let mut value = self.states[index]
            .try_borrow_mut()
            .context("value already borrowed")?;

        self.initialize_value::<T>(cache_key, &mut value, params)?;
        let value = transpose_ref_mut_opt(value).context("expected a valid value")?;

        Ok(RefMut::map(value, |value| {
            value.value.downcast_mut::<T>().expect("to be of type T")
        }))
    }

    pub fn resolve<T: State>(&self, params: T::Parameter) -> anyhow::Result<Ref<'_, T>> {
        let (cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T>(&params, true)
            .context("state capacity exceeded")?;

        if let Ok(mut value) = self.states[index].try_borrow_mut() {
            self.initialize_value::<T>(cache_key, &mut value, params)?;
        } else {
            /* We already borrowed that state, hence it must be initialized & not dirty */
        }

        let value = self.states[index].try_borrow().map_err(|_| {
            anyhow!(
                "circular state initialisation for {}",
                any::type_name::<T>()
            )
        })?;

        let value = Ref::map(value, |value| {
            let value = value.as_ref().expect("to be present");
            value.value.downcast_ref::<T>().expect("to be of type T")
        });
        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use super::{
        State,
        StateCacheType,
        StateRegistry,
    };

    struct StateA;
    impl State for StateA {
        type Parameter = ();

        fn create(_states: &StateRegistry, _params: Self::Parameter) -> anyhow::Result<Self> {
            println!("State A created");
            Ok(Self)
        }

        fn cache_type() -> StateCacheType {
            StateCacheType::Volatile
        }
    }

    struct StateB;
    impl State for StateB {
        type Parameter = ();

        fn create(_states: &StateRegistry, _params: Self::Parameter) -> anyhow::Result<Self> {
            println!("State B created");
            Ok(Self)
        }

        fn cache_type() -> StateCacheType {
            StateCacheType::Persistent
        }
    }

    struct StateC;
    impl State for StateC {
        type Parameter = u64;

        fn create(states: &StateRegistry, params: Self::Parameter) -> anyhow::Result<Self> {
            assert!(states.resolve::<StateA>(()).is_ok());
            println!("State C({}) created", params);
            assert!(states.resolve::<StateB>(()).is_ok());
            if params == 1 {
                assert!(states.resolve::<StateC>(1).is_err());
            } else {
                assert!(states.resolve::<StateC>(1).is_ok());
            }
            Ok(Self)
        }

        fn cache_type() -> StateCacheType {
            StateCacheType::Persistent
        }
    }

    #[test]
    fn test_creation_0() {
        let states = StateRegistry::new(10);
        assert!(states.resolve::<StateA>(()).is_ok());
    }

    #[test]
    fn test_creation_1() {
        let states = StateRegistry::new(4);
        assert!(states.resolve::<StateC>(0).is_ok());
    }

    #[test]
    fn test_expire() {
        let mut states = StateRegistry::new(2);
        assert!(states.resolve::<StateA>(()).is_ok());
        assert!(states.resolve::<StateB>(()).is_ok());
        states.invalidate_states();
        assert!(states.get::<StateA>(()).is_none());
        assert!(states.get::<StateB>(()).is_some());
        assert!(states.resolve::<StateA>(()).is_ok());
        assert!(states.resolve::<StateB>(()).is_ok());
        assert!(states.get::<StateA>(()).is_some());
        assert!(states.get::<StateB>(()).is_some());
    }
}
