//! Interrupt Request Level (IRQL).

/// IRQL type.
pub type KIRQL = u8;

/// Passive release level, no interrupt vectors are masked.
pub const PASSIVE_LEVEL: KIRQL = 0;
/// The lowest IRQL level, no interrupt vectors are masked.
pub const LOW_LEVEL: KIRQL = 0;
/// APC interrupt level.
pub const APC_LEVEL: KIRQL = 1;
/// Dispatcher level
pub const DISPATCH_LEVEL: KIRQL = 2;

/// Timer used for profiling.
#[cfg(target_arch = "x86")]
pub const PROFILE_LEVEL: KIRQL = 27;

/// Interval clock level.
#[cfg(target_arch = "x86")]
pub const CLOCK_LEVEL: KIRQL = 28;

/// Interprocessor interrupt level.
#[cfg(target_arch = "x86")]
pub const IPI_LEVEL: KIRQL = 29;

/// Power failure level.
#[cfg(target_arch = "x86")]
pub const POWER_LEVEL: KIRQL = 30;

/// Highest interrupt level.
#[cfg(target_arch = "x86")]
pub const HIGH_LEVEL: KIRQL = 31;

/// Synchronization level.
#[cfg(target_arch = "x86")]
pub const SYNCH_LEVEL: KIRQL = 29 - 2;

/// Interval clock level.
#[cfg(target_arch = "x86_64")]
pub const CLOCK_LEVEL: KIRQL = 13;

/// Interprocessor interrupt level.
#[cfg(target_arch = "x86_64")]
pub const IPI_LEVEL: KIRQL = 14;

/// Power failure level.
#[cfg(target_arch = "x86_64")]
pub const POWER_LEVEL: KIRQL = 15;

/// Timer used for profiling.
#[cfg(target_arch = "x86_64")]
pub const PROFILE_LEVEL: KIRQL = 16;

/// Highest interrupt level.
#[cfg(target_arch = "x86_64")]
pub const HIGH_LEVEL: KIRQL = 17;

/// Synchronization level.
#[cfg(target_arch = "x86_64")]
pub const SYNCH_LEVEL: KIRQL = 14- 2;
