//! Canonical typed serialization for authored semantic identity. Map iteration is
//! irrelevant; execution sequence order and exact finite numeric values are not.
//! This representation is hashed, never deserialized into a gameplay definition.

use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use serde_json::{json, Value};

type Error = serde_json::Error;
type Result<T> = std::result::Result<T, Error>;

pub(crate) fn bytes(value: &impl Serialize) -> Result<Vec<u8>> {
    serde_json::to_vec(&value.serialize(Canonical)?)
}

struct Canonical;

fn tagged(tag: &str, value: Value) -> Value {
    json!([tag, value])
}

impl ser::Serializer for Canonical {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = Sequence;
    type SerializeTuple = Sequence;
    type SerializeTupleStruct = Sequence;
    type SerializeTupleVariant = Sequence;
    type SerializeMap = Mapping;
    type SerializeStruct = Mapping;
    type SerializeStructVariant = Mapping;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(v.into())
    }
    fn serialize_i8(self, v: i8) -> Result<Value> {
        self.serialize_i64(v.into())
    }
    fn serialize_i16(self, v: i16) -> Result<Value> {
        self.serialize_i64(v.into())
    }
    fn serialize_i32(self, v: i32) -> Result<Value> {
        self.serialize_i64(v.into())
    }
    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(tagged("signed", v.into()))
    }
    fn serialize_i128(self, v: i128) -> Result<Value> {
        Ok(tagged("i128", v.to_string().into()))
    }
    fn serialize_u8(self, v: u8) -> Result<Value> {
        self.serialize_u64(v.into())
    }
    fn serialize_u16(self, v: u16) -> Result<Value> {
        self.serialize_u64(v.into())
    }
    fn serialize_u32(self, v: u32) -> Result<Value> {
        self.serialize_u64(v.into())
    }
    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(tagged("unsigned", v.into()))
    }
    fn serialize_u128(self, v: u128) -> Result<Value> {
        Ok(tagged("u128", v.to_string().into()))
    }
    fn serialize_f32(self, v: f32) -> Result<Value> {
        if !v.is_finite() {
            return Err(ser::Error::custom("nonfinite authored f32"));
        }
        Ok(tagged("f32", format!("{:08x}", v.to_bits()).into()))
    }
    fn serialize_f64(self, v: f64) -> Result<Value> {
        if !v.is_finite() {
            return Err(ser::Error::custom("nonfinite authored f64"));
        }
        Ok(tagged("f64", format!("{:016x}", v.to_bits()).into()))
    }
    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(tagged("char", v.to_string().into()))
    }
    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(v.into())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        Ok(tagged("bytes", json!(v)))
    }
    fn serialize_none(self) -> Result<Value> {
        Ok(json!(["none"]))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<Value> {
        Ok(tagged("some", v.serialize(self)?))
    }
    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Value> {
        Ok(json!(["unit", name]))
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Value> {
        Ok(json!(["enum", name, variant, Value::Null]))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        v: &T,
    ) -> Result<Value> {
        Ok(json!(["newtype", name, v.serialize(self)?]))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<Value> {
        Ok(json!(["enum", name, variant, v.serialize(self)?]))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Sequence> {
        Ok(Sequence::new(vec![json!("seq")]))
    }
    fn serialize_tuple(self, _: usize) -> Result<Sequence> {
        Ok(Sequence::new(vec![json!("tuple")]))
    }
    fn serialize_tuple_struct(self, name: &'static str, _: usize) -> Result<Sequence> {
        Ok(Sequence::new(vec![json!("tuple_struct"), json!(name)]))
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Sequence> {
        Ok(Sequence::new(vec![
            json!("enum"),
            json!(name),
            json!(variant),
        ]))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Mapping> {
        Ok(Mapping::new(vec![json!("map")]))
    }
    fn serialize_struct(self, name: &'static str, _: usize) -> Result<Mapping> {
        Ok(Mapping::new(vec![json!("struct"), json!(name)]))
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Mapping> {
        Ok(Mapping::new(vec![
            json!("enum"),
            json!(name),
            json!(variant),
        ]))
    }
}

struct Sequence {
    prefix: Vec<Value>,
    values: Vec<Value>,
}

impl Sequence {
    fn new(prefix: Vec<Value>) -> Self {
        Self {
            prefix,
            values: vec![],
        }
    }
    fn push<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.values.push(v.serialize(Canonical)?);
        Ok(())
    }
    fn finish(mut self) -> Result<Value> {
        self.prefix.push(Value::Array(self.values));
        Ok(Value::Array(self.prefix))
    }
}

macro_rules! sequence_impl {
    ($trait:ident, $method:ident) => {
        impl $trait for Sequence {
            type Ok = Value;
            type Error = Error;
            fn $method<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
                self.push(v)
            }
            fn end(self) -> Result<Value> {
                self.finish()
            }
        }
    };
}
sequence_impl!(SerializeSeq, serialize_element);
sequence_impl!(SerializeTuple, serialize_element);
sequence_impl!(SerializeTupleStruct, serialize_field);
sequence_impl!(SerializeTupleVariant, serialize_field);

struct Mapping {
    prefix: Vec<Value>,
    entries: Vec<(Vec<u8>, Value, Value)>,
    pending_key: Option<Value>,
}

impl Mapping {
    fn new(prefix: Vec<Value>) -> Self {
        Self {
            prefix,
            entries: vec![],
            pending_key: None,
        }
    }
    fn field<T: ?Sized + Serialize>(&mut self, key: &'static str, v: &T) -> Result<()> {
        self.serialize_entry(key, v)
    }
    fn finish(mut self) -> Result<Value> {
        if self.pending_key.is_some() {
            return Err(ser::Error::custom("map key without value"));
        }
        self.entries.sort_by(|a, b| a.0.cmp(&b.0));
        if self.entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(ser::Error::custom("duplicate canonical map key"));
        }
        self.prefix.push(Value::Array(
            self.entries
                .into_iter()
                .map(|(_, k, v)| json!([k, v]))
                .collect(),
        ));
        Ok(Value::Array(self.prefix))
    }
}

impl SerializeMap for Mapping {
    type Ok = Value;
    type Error = Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        if self.pending_key.is_some() {
            return Err(ser::Error::custom("map key without value"));
        }
        self.pending_key = Some(key.serialize(Canonical)?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| <Error as ser::Error>::custom("map value without key"))?;
        self.entries
            .push((serde_json::to_vec(&key)?, key, v.serialize(Canonical)?));
        Ok(())
    }
    fn end(self) -> Result<Value> {
        self.finish()
    }
}

macro_rules! mapping_impl {
    ($trait:ident) => {
        impl $trait for Mapping {
            type Ok = Value;
            type Error = Error;
            fn serialize_field<T: ?Sized + Serialize>(
                &mut self,
                key: &'static str,
                v: &T,
            ) -> Result<()> {
                self.field(key, v)
            }
            fn end(self) -> Result<Value> {
                self.finish()
            }
        }
    };
}
mapping_impl!(SerializeStruct);
mapping_impl!(SerializeStructVariant);

#[cfg(test)]
mod tests {
    use super::bytes;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn unordered_trigger_maps_are_stable_but_execution_sequences_are_not_sorted() {
        let mut a = HashMap::new();
        a.insert("on_hit", vec![1, 2]);
        a.insert("on_cast", vec![3]);
        let mut b = HashMap::new();
        b.insert("on_cast", vec![3]);
        b.insert("on_hit", vec![1, 2]);
        let ordered: BTreeMap<_, _> = a.iter().map(|(k, v)| (*k, v.clone())).collect();
        assert_eq!(bytes(&a).unwrap(), bytes(&b).unwrap());
        assert_eq!(bytes(&a).unwrap(), bytes(&ordered).unwrap());
        b.get_mut("on_hit").unwrap().reverse();
        assert_ne!(bytes(&a).unwrap(), bytes(&b).unwrap());
    }

    #[test]
    fn nonfinite_nested_values_cannot_collapse_to_optional_null() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(bytes(&vec![Some(value)])
                .unwrap_err()
                .to_string()
                .contains("nonfinite"));
        }
        assert!(bytes(&Some(f32::INFINITY)).is_err());
        assert_ne!(bytes(&Some(0.0_f64)).unwrap(), bytes(&None::<f64>).unwrap());
        assert_ne!(bytes(&0.0_f64).unwrap(), bytes(&(-0.0_f64)).unwrap());
    }

    #[test]
    fn enum_and_option_identity_is_preserved_without_compiled_sources() {
        #[derive(serde::Serialize)]
        enum Op {
            Damage(u32),
            Heal(u32),
        }
        #[derive(serde::Serialize)]
        struct Effect {
            op: Op,
            #[serde(skip)]
            source: String,
        }
        let a = Effect {
            op: Op::Damage(7),
            source: "a.ron".into(),
        };
        let b = Effect {
            op: Op::Damage(7),
            source: "b.ron".into(),
        };
        assert_ne!(a.source, b.source);
        assert_eq!(bytes(&a).unwrap(), bytes(&b).unwrap());
        assert_ne!(bytes(&a.op).unwrap(), bytes(&Op::Heal(7)).unwrap());
        assert_ne!(bytes(&Some(Some(7))).unwrap(), bytes(&Some(7)).unwrap());
    }
}
