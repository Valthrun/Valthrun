use std::{
    collections::BTreeMap,
    sync::Arc,
};

use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    CS2Handle,
};
use cs2_schema_declaration::Ptr;
use cs2_schema_generated::cs2::client::CEntityIdentity;

pub struct ClassNameCache {
    cs2: Arc<CS2Handle>,

    lookup: BTreeMap<u64, String>,
    reverse_lookup: BTreeMap<String, u64>,
}

impl ClassNameCache {
    pub fn new(cs2: Arc<CS2Handle>) -> Self {
        Self {
            cs2,

            lookup: Default::default(),
            reverse_lookup: Default::default(),
        }
    }

    pub fn update_cache(&mut self, known_identities: &[CEntityIdentity]) -> anyhow::Result<()> {
        for identity in known_identities {
            self.register_class_info(identity.entity_class_info()?)
                .with_context(|| {
                    format!(
                        "failed to generate class info for entity {:?}",
                        identity.handle::<()>().unwrap_or_default()
                    )
                })?;
        }

        Ok(())
    }

    fn register_class_info(&mut self, class_info: Ptr<()>) -> anyhow::Result<()> {
        let address = class_info.address()?;
        if self.lookup.contains_key(&address) {
            /* we already know the name for this class */
            return Ok(());
        }

        let class_name = self
            .cs2
            .read_string(&[address + 0x28, 0x08, 0x00], Some(32))?;

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
