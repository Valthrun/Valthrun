use std::collections::{
    BTreeMap,
    HashSet,
};

use crate::{
    mod_name_from_schema_name,
    SchemaScope,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClassReference {
    pub module_name: String,
    pub class_name: String,
}

impl ClassReference {
    pub fn from_rs_path(name: &str) -> Option<Self> {
        let (module_name, class_name) = name.split_once("::")?;
        Some(Self {
            class_name: class_name.to_string(),
            module_name: module_name.to_string(),
        })
    }
}

pub struct InheritageMap {
    mapping: BTreeMap<ClassReference, ClassReference>,
}

impl InheritageMap {
    pub fn build(src: &[SchemaScope]) -> Self {
        let mut mapping = BTreeMap::default();

        for scope in src.iter() {
            for class in scope.classes.iter() {
                let Some(inherits) = &class.inherits else {
                    continue;
                };

                let reference = ClassReference {
                    class_name: class.class_name.clone(),
                    module_name: mod_name_from_schema_name(&scope.schema_name).to_string(),
                };

                if let Some(inherits) = ClassReference::from_rs_path(&inherits) {
                    mapping.insert(reference, inherits);
                } else {
                    println!("Invalid class {}", inherits);
                }
            }
        }

        Self { mapping }
    }

    pub fn get_inherited_classes(&self, reference: &ClassReference) -> HashSet<ClassReference> {
        let mut result = HashSet::new();

        let mut open_list = Vec::with_capacity(8);
        open_list.push(reference);
        while let Some(entry) = open_list.pop() {
            let Some(inherited_class) = self.mapping.get(entry) else {
                continue;
            };

            if !result.insert(inherited_class.clone()) {
                /* class already visited */
                continue;
            }

            open_list.push(inherited_class);
        }

        result
    }
}
