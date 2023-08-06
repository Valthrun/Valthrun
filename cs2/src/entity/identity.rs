use cs2_schema::{SchemaValue, Ptr, EntityHandle, cs2::client::CEntityIdentity};

pub trait CEntityIdentityEx {
    fn entity_ptr<T>(&self) -> anyhow::Result<Ptr<T>>;
    fn entity_vtable(&self) -> anyhow::Result<Ptr<Ptr<()>>>;
    fn handle<T>(&self) -> anyhow::Result<EntityHandle<T>>; 
}

impl CEntityIdentityEx for CEntityIdentity {
    fn entity_ptr<T>(&self) -> anyhow::Result<Ptr<T>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    /// Returns a ptr ptr to the entities vtable.
    /// The first pointer might be null, if the entity identity is invalid.
    fn entity_vtable(&self) -> anyhow::Result<Ptr<Ptr<()>>> {
        Ok(self.entity_ptr::<()>()?.cast::<Ptr<()>>())
    }

    fn handle<T>(&self) -> anyhow::Result<EntityHandle<T>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x10)
    }
}