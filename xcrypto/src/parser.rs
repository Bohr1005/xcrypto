use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Debug)]
pub struct Parser {
    value: Value,
}

impl Parser {
    pub fn new(s: &str) -> anyhow::Result<Self> {
        Ok({
            Self {
                value: serde_json::from_str(s)?,
            }
        })
    }

    pub fn contains(&self, key: &str) -> bool {
        self.value.get(key).is_some()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.value.get(key)
    }

    pub fn decode<T>(&self) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(T::deserialize(&self.value)?)
    }

    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }
}

pub fn decode<T>(value: Value) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    Ok(T::deserialize(value)?)
}

impl From<Value> for Parser {
    fn from(value: Value) -> Self {
        Self { value }
    }
}
