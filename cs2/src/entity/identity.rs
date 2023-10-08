use cs2_schema_declaration::Ptr;
use cs2_schema_generated::{
    cs2::client::{
        CEntityIdentity,
        CEntityInstance,
    },
    EntityHandle,
};

pub trait CEntityIdentityEx {
    fn entity_ptr<T>(&self) -> anyhow::Result<Ptr<T>>;
    fn entity_class_info(&self) -> anyhow::Result<Ptr<()>>;

    fn handle<T>(&self) -> anyhow::Result<EntityHandle<T>>;
}

impl CEntityIdentityEx for CEntityIdentity {
    fn entity_ptr<T>(&self) -> anyhow::Result<Ptr<T>> {
        self.memory.reference_schema(0x00)
    }

    /// Returns a ptr to the entity runtime info
    fn entity_class_info(&self) -> anyhow::Result<Ptr<()>> {
        self.memory.reference_schema(0x08)
    }

    fn handle<T>(&self) -> anyhow::Result<EntityHandle<T>> {
        self.memory.reference_schema(0x10)
    }
}

pub trait CEntityInstanceEx {
    fn vtable(&self) -> anyhow::Result<Ptr<()>>;
}

impl CEntityInstanceEx for CEntityInstance {
    fn vtable(&self) -> anyhow::Result<Ptr<()>> {
        self.memory.reference_schema(0x00)
    }
}
