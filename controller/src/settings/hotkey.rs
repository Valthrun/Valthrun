use serde::{
    de::Visitor,
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug)]
pub struct HotKey(pub imgui::Key);

impl From<imgui::Key> for HotKey {
    fn from(value: imgui::Key) -> Self {
        Self(value)
    }
}

impl Serialize for HotKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self.0))
    }
}

struct HotKeyVisitor;

impl<'de> Visitor<'de> for HotKeyVisitor {
    type Value = HotKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a config key")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        for key in imgui::Key::VARIANTS.iter() {
            if format!("{:?}", key) == v {
                return Ok(HotKey(key.clone()));
            }
        }

        Err(E::custom("unknown key value"))
    }
}

impl<'de> Deserialize<'de> for HotKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HotKeyVisitor)
    }
}
