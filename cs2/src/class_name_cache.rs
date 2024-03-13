use std::collections::BTreeMap;

use anyhow::Context;
use cs2_schema_declaration::Ptr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    CS2Handle,
    CS2HandleState,
    EntitySystem,
};

pub struct ClassNameCache {
    lookup: BTreeMap<u64, String>,
    reverse_lookup: BTreeMap<String, u64>,
}

impl State for ClassNameCache {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            lookup: Default::default(),
            reverse_lookup: Default::default(),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let entities = states.resolve::<EntitySystem>(())?;
        for identity in entities.all_identities() {
            self.register_class_info(&cs2, identity.entity_class_info()?)
                .with_context(|| {
                    format!(
                        "failed to generate class info for entity {:?}",
                        identity.handle::<()>().unwrap_or_default()
                    )
                })?;
        }
        Ok(())
    }
}

impl ClassNameCache {
    fn register_class_info(&mut self, cs2: &CS2Handle, class_info: Ptr<()>) -> anyhow::Result<()> {
        let address = class_info.address()?;
        if self.lookup.contains_key(&address) {
            /* we already know the name for this class */
            return Ok(());
        }

        let class_name = cs2.read_string(&[address + 0x28, 0x08, 0x00], Some(32))?;
        self.lookup.insert(address, class_name.clone());
        self.reverse_lookup.insert(class_name, address);
        Ok(())
    }

    pub fn lookup(&self, class_info: &Ptr<()>) -> anyhow::Result<Option<&String>> {
        let address = class_info.address()?;
        Ok(self.lookup.get(&address))
    }

    pub fn reverse_lookup(&self, name: &str) -> Option<u64> {
        self.reverse_lookup.get(name).cloned()
    }
}
