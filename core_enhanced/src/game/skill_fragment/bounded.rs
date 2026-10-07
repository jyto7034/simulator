//! Reject duplicate keys and excess entries before decoding another value.
use serde::{
    de::{Error, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};

pub(crate) fn map<'de, D, K, V, const LIMIT: usize>(
    deserializer: D,
) -> Result<BTreeMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
{
    struct Unique<K, V, const N: usize>(PhantomData<(K, V)>);
    impl<'de, K: Deserialize<'de> + Ord, V: Deserialize<'de>, const N: usize> Visitor<'de>
        for Unique<K, V, N>
    {
        type Value = BTreeMap<K, V>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "a unique map with at most {N} entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
            let mut output = BTreeMap::new();
            while let Some(key) = input.next_key()? {
                if output.len() >= N {
                    return Err(A::Error::custom("fragment map limit"));
                }
                if output.contains_key(&key) {
                    return Err(A::Error::custom("duplicate fragment key"));
                }
                output.insert(key, input.next_value()?);
            }
            Ok(output)
        }
    }
    deserializer.deserialize_map(Unique::<K, V, LIMIT>(PhantomData))
}
pub(crate) fn forms<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeSet<String>, D::Error> {
    struct Unique;
    impl<'de> Visitor<'de> for Unique {
        type Value = BTreeSet<String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("at most 16 distinct fragment form IDs")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
            let mut output = BTreeSet::new();
            while let Some(key) = input.next_element::<String>()? {
                if output.len() >= super::MAX_FRAGMENT_FORMS
                    || key.is_empty()
                    || key.len() > 64
                    || !key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                {
                    return Err(A::Error::custom("fragment form limit or invalid ID"));
                }
                if !output.insert(key) {
                    return Err(A::Error::custom("duplicate fragment form"));
                }
            }
            Ok(output)
        }
    }
    deserializer.deserialize_seq(Unique)
}
