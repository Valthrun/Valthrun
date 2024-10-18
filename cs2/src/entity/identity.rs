use anyhow::anyhow;
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::{
    CEntityIdentity,
    CEntityInstance,
};
use raw_struct::{
    builtins::Ptr64,
    raw_struct,
    FromMemoryView,
    Viewable,
};

pub trait CEntityIdentityEx {
    fn entity_ptr<T: ?Sized>(&self) -> anyhow::Result<Ptr64<T>>;
    fn entity_class_info(&self) -> anyhow::Result<Ptr64<()>>;

    fn handle<T: ?Sized>(&self) -> anyhow::Result<EntityHandle<T>>;
}

impl CEntityIdentityEx for dyn CEntityIdentity {
    fn entity_ptr<T: ?Sized>(&self) -> anyhow::Result<Ptr64<T>> {
        Ptr64::read_object(self.object_memory(), 0x00).map_err(|e| anyhow!(e))
    }

    fn entity_class_info(&self) -> anyhow::Result<Ptr64<()>> {
        Ptr64::read_object(self.object_memory(), 0x08).map_err(|e| anyhow!(e))
    }

    fn handle<T: ?Sized>(&self) -> anyhow::Result<EntityHandle<T>> {
        EntityHandle::read_object(self.object_memory(), 0x10).map_err(|e| anyhow!(e))
    }
}

pub trait CEntityInstanceEx {
    fn vtable(&self) -> anyhow::Result<Ptr64<()>>;
}

impl CEntityInstanceEx for dyn CEntityInstance {
    fn vtable(&self) -> anyhow::Result<Ptr64<()>> {
        Ptr64::read_object(self.object_memory(), 0x00).map_err(|e| anyhow!(e))
    }
}

#[raw_struct(size = "<dyn CEntityIdentity as Viewable<_>>::MEMORY_SIZE")]
pub struct TypedEntityIdentity<T>
where
    T: ?Sized + Send + Sync + 'static, {}

impl<T: ?Sized> CEntityIdentity for dyn TypedEntityIdentity<T> {}

impl<T: ?Sized> dyn TypedEntityIdentity<T> {
    pub fn entity(&self) -> anyhow::Result<Ptr64<T>> {
        Ptr64::read_object(self.object_memory(), 0x00).map_err(|e| anyhow!(e))
    }
}
