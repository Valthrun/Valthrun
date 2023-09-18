use cs2_schema_declaration::Ptr;
use cs2_schema_generated::{
    cs2::client::{CEntityIdentity, CEntityInstance},
    EntityHandle,
};

pub trait CEntityIdentityEx {
    fn entity_ptr<T>(&self) -> anyhow::Result<Ptr<T>>;
    fn entity_vtable(&self) -> anyhow::Result<Ptr<Ptr<()>>>;
    fn handle<T>(&self) -> anyhow::Result<EntityHandle<T>>;
}

impl CEntityIdentityEx for CEntityIdentity {
    fn entity_ptr<T>(&self) -> anyhow::Result<Ptr<T>> {
        self.memory.reference_schema(0x00)
    }

    /// Returns a ptr ptr to the entities vtable.
    /// The first pointer might be null, if the entity identity is invalid.
    fn entity_vtable(&self) -> anyhow::Result<Ptr<Ptr<()>>> {
        Ok(self.entity_ptr::<()>()?.cast::<Ptr<()>>())
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
