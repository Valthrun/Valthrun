//! Fast mutex implementation from https://github.com/StephanvanSchaik/windows-kernel-rs (MIT license)

use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use crate::kdef::{_FAST_MUTEX, ExInitializeFastMutex, ExTryToAcquireFastMutex, ExAcquireFastMutex, ExReleaseFastMutex};

/// A mutual exclusion primitive useful for protecting shared data.
///
/// This mutex will block threads waiting for the lock to become available. The mutex can also be
/// statically initialized or created via a [`new`] constructor. Each mutex has a type parameter
/// which represents the data that it is protecting. The data can only be accessed through the RAII
/// guards returned from [`lock`] and [`try_lock`], which guarantees that the data is only ever
/// accessed when the mutex is locked.
///
/// [`new`]: FastMutex::new
/// [`lock`]: FastMutex::lock
/// [`try_lock`]: FastMutex::try_lock
pub struct FastMutex<T: ?Sized> {
    lock: Box<UnsafeCell<_FAST_MUTEX>>,
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for FastMutex<T> {}
unsafe impl<T> Sync for FastMutex<T> {}

impl<T> FastMutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    pub fn new(data: T) -> Self {
        let lock = Box::new(unsafe {
            UnsafeCell::new(core::mem::zeroed())
        });

        unsafe {
            ExInitializeFastMutex(
                &mut *lock.get(),
            )
        };

        Self {
            lock,
            data: UnsafeCell::new(data),
        }
    }

    /// Consumes this `FastMutex`, returning the underlying data.
    #[inline]
    pub fn into_inner(self) -> T {
        let Self { data, .. } = self;
        data.into_inner()
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned. Otherwise, an RAII
    /// guard is returned. The lock will be unlocked when the guard is dropped.
    ///
    /// This function does not block.
    #[inline]
    pub fn try_lock(&self) -> Option<FastMutexGuard<T>> {
        let status = unsafe {
            ExTryToAcquireFastMutex(
                self.lock.get()
            )
        } != 0;

        match status {
            true => Some(FastMutexGuard {
                lock: unsafe { &mut *self.lock.get() },
                data: unsafe { &mut *self.data.get() },
            }),
            _ => None,
        }
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire the mutex. Upon
    /// returning, the thread is the only thread with the lock held. An RAII guard is returned to
    /// allow scoped unlock of the lock. When the guard goes out of scope, the mutex will be
    /// unlocked.
    ///
    /// The underlying function does not allow for recursion. If the thread already holds the lock
    /// and tries to lock the mutex again, this function will return `None` instead.
    #[inline]
    pub fn lock(&self) -> FastMutexGuard<T> {
        unsafe {
            ExAcquireFastMutex(
                &mut *self.lock.get(),
            )
        };

        FastMutexGuard {
            lock: unsafe { &mut *self.lock.get() },
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl<T: ?Sized + Default> Default for FastMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for FastMutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is dropped (falls out
/// of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its [`Deref`] and
/// [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on [`FastMutex`].
///
/// [`lock`]: FastMutex::lock
/// [`try_lock`]: FastMutex::try_lock
pub struct FastMutexGuard<'a, T: 'a + ?Sized> {
    lock: &'a mut _FAST_MUTEX,
    data: &'a mut T,
}

impl<'a, T: ?Sized> Drop for FastMutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ExReleaseFastMutex(
                &mut *self.lock,
            )
        };
    }
}

impl<'a, T: ?Sized> Deref for FastMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for FastMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}