//! Event Objects.

use super::DISPATCHER_HEADER;

extern "system"
{
	pub fn KeInitializeEvent(Event: PKEVENT, Type: EVENT_TYPE, State: bool);
	pub fn KeSetEvent(Event: PKEVENT, Increment: i32, Wait: bool) -> i32;
	pub fn KeReadStateEvent(Event: PKEVENT) -> i32;
	pub fn KeResetEvent(Event: PKEVENT) -> i32;
	pub fn KeClearEvent(Event: PKEVENT);
}

pub type PKEVENT = *mut KEVENT;

/// Specifies the event type.
#[repr(C)]
pub enum EVENT_TYPE
{
	/// Manual-reset event.
	NotificationEvent = 0,
	/// Auto-clearing event.
	SynchronizationEvent,
}

/// Event object.
#[repr(C)]
pub struct KEVENT
{
	Header: DISPATCHER_HEADER,
}
