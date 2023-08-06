use cs2_schema::PtrCStr;

use crate::{CS2Handle, Module};

pub trait PCStrEx {
    fn read_string(&self, cs2: &CS2Handle) -> anyhow::Result<String>;
    fn try_read_string(&self, cs2: &CS2Handle) -> anyhow::Result<Option<String>>;
}

impl PCStrEx for PtrCStr {
    fn read_string(&self, cs2: &CS2Handle) -> anyhow::Result<String> {
        cs2.read_string(Module::Absolute, &[ self.address()? ], None)
    }

    fn try_read_string(&self, cs2: &CS2Handle) -> anyhow::Result<Option<String>> {
        let address = self.address()?;
        if address == 0 {
            Ok(None)
        } else {
            Ok(Some(cs2.read_string(
                Module::Absolute,
                &[ address ],
                None,
            )?))
        }
    }
}