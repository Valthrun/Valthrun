use std::cell::SyncUnsafeCell;

/// Lazy offset which will be resolved as soon it's first used.
/// The result will be cached. Multiple requests can occurr at once!
pub trait LazyOffset: Sized + Clone + Send {
    /// Resolve the target offset.
    fn offset(self) -> anyhow::Result<u64>;
}

impl LazyOffset for u64 {
    fn offset(self) -> anyhow::Result<u64> {
        Ok(self)
    }
}

#[derive(Clone)]
enum CachedOffsetState {
    Unresolved,
    Resolved(u64),
}

pub struct CachedOffset {
    state: SyncUnsafeCell<CachedOffsetState>,
}

impl CachedOffset {
    pub const fn new() -> Self {
        Self {
            state: SyncUnsafeCell::new(CachedOffsetState::Unresolved),
        }
    }

    pub fn resolve<T: LazyOffset>(&self, resolver: impl Fn() -> T) -> anyhow::Result<u64> {
        let state = unsafe { (&*self.state.get()).clone() };

        match state {
            CachedOffsetState::Resolved(offset) => Ok(offset),
            CachedOffsetState::Unresolved => {
                let offset = resolver().offset()?;
                unsafe {
                    *self.state.get() = CachedOffsetState::Resolved(offset);
                }

                Ok(offset)
            }
        }
    }
}
