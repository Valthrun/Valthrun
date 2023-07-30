pub type _OBJECT_TYPE = ();
pub type POBJECT_TYPE = *const _OBJECT_TYPE;

#[allow(unused)]
extern "system" {
	pub static CmKeyObjectType: *const POBJECT_TYPE;
	pub static IoFileObjectType: *const POBJECT_TYPE;
	pub static ExEventObjectType: *const POBJECT_TYPE;
	pub static ExSemaphoreObjectType: *const POBJECT_TYPE;
	pub static TmTransactionManagerObjectType: *const POBJECT_TYPE;
	pub static TmResourceManagerObjectType: *const POBJECT_TYPE;
	pub static TmEnlistmentObjectType: *const POBJECT_TYPE;
	pub static TmTransactionObjectType: *const POBJECT_TYPE;
	pub static PsProcessType: *const POBJECT_TYPE;
	pub static PsThreadType: *const POBJECT_TYPE;
	pub static PsJobType: *const POBJECT_TYPE;
	pub static SeTokenObjectType: *const POBJECT_TYPE;
}
