use std::io::Read;

use crate::parser::Map;

pub use super::{Error, Result};
use serde::{Deserialize, de::{DeserializeOwned, IntoDeserializer}};
use super::parser::{Parser, Value};

struct MapAccessor {
    value: Option<Value>,
    iter: std::collections::hash_map::IntoIter<String, Value>
}

impl From<Map> for MapAccessor {
    fn from(value: Map) -> Self {
        Self {
            value: None,
            iter: value.into_iter()
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for MapAccessor {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de> {
        match self.iter.next() {
            Some((k, v)) => {
                let key = seed.deserialize(FormDecoder(k.into()))?;
                self.value = Some(v);
                return Ok(Some(key))
            },
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de> {
        match std::mem::take(&mut self.value) {
            Some(value) => {
                seed.deserialize(FormDecoder(value))
            },
            None => Err(Error::MissingMapValue),
        }
    }    
}

struct SeqAccessor {
    index: usize,
    len: usize,
    map: Map
}

impl<'de> serde::de::SeqAccess<'de> for SeqAccessor {
    type Error = Error;
   
    fn next_element_seed<T>(&mut self, seed: T) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
        if self.index >= self.len { return Ok(None) }
        
        let (_, v) = self.map.take(&self.index.to_string()).ok_or_else(|| Error::MissingSequenceItem)?;
        self.index += 1;
        
        let value = seed.deserialize(FormDecoder(v))?;
        Ok(Some(value))
    }
}

impl TryFrom<Map> for SeqAccessor {
    type Error = Error;

    fn try_from(map: Map) -> Result<Self> {
        let len: usize = map.borrow(&["$length"])
            .ok_or_else(|| Error::MissingSequenceLength)?
            .try_as_ref_single()
            .ok_or_else(|| Error::ExpectingString)?
            .parse::<usize>()
            .map_err(|_| Error::ExpectingUsize)?;

        Ok(Self {
            index: 0,
            len,
            map
        })

    }
}

pub struct FormDecoder(Value);

impl FormDecoder {
    fn new(input: &str) -> Result<Self> {
        let parser = Parser::new(input);
        let value = parser.collect::<Result<Value>>()?;
        Ok(Self(value))
    }
}

impl FormDecoder {
    fn try_as_single(self) -> Result<String> {
        self.0.try_as_single().ok_or(Error::ExpectingString)
    }

    fn try_as_map(self) -> Result<Map> {
        self.0.try_as_map().ok_or(Error::ExpectingMap)
    }
}

impl<'de> serde::Deserializer<'de> for FormDecoder {
    type Error = Error;

    fn deserialize_struct<V>(self, _name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        
        let map_access = MapAccessor::from(self.try_as_map()?);
        visitor.visit_map(map_access)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        let single = self.try_as_single()?;
        visitor.visit_enum(single.into_deserializer())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_str(&self.try_as_single()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        self.deserialize_any(visitor)
    }
    
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        match &self.0 {
            Value::Single(_) => self.deserialize_str(visitor),
            Value::Map(_) => self.deserialize_map(visitor),
        }
    }
    
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        match self.try_as_single()?.to_lowercase().as_str() {
            "false" => visitor.visit_bool(false),
            "true" => visitor.visit_bool(true),
            _ => Err(Error::ExpectingBool)
        }
    }
    
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_i8(
            self.try_as_single()?
                .parse::<i8>()
                .map_err(|_| Error::ExpectingI8)?
        )
    }
    
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_i16(
            self.try_as_single()?
                .parse::<i16>()
                .map_err(|_| Error::ExpectingI16)?
        )
    }
    
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_i32(
            self.try_as_single()?
                .parse::<i32>()
                .map_err(|_| Error::ExpectingI32)?
        )
    }
    
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_i64(
            self.try_as_single()?
                .parse::<i64>()
                .map_err(|_| Error::ExpectingI64)?
        )
    }
    
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_u8(
            self.try_as_single()?
                .parse::<u8>()
                .map_err(|_| Error::ExpectingU8)?
        )
    }
    
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_u16(
            self.try_as_single()?
                .parse::<u16>()
                .map_err(|_| Error::ExpectingU16)?
        )
    }
    
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_u32(
            self.try_as_single()?
                .parse::<u32>()
                .map_err(|_| Error::ExpectingU32)?
        )
    }
    
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_u64(
            self.try_as_single()?
                .parse::<u64>()
                .map_err(|_| Error::ExpectingU64)?
        )
    }
    
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_f32(
            self.try_as_single()?
                .parse::<f32>()
                .map_err(|_| Error::ExpectingF32)?
        )
    }
    
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_f64(
            self.try_as_single()?
                .parse::<f64>()
                .map_err(|_| Error::ExpectingF64)?
        )
    }
    
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_char(self.try_as_single()?.chars().next().ok_or_else(|| Error::ExpectingChar)?)
    }
    
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_str(&self.try_as_single()?)
    }
    
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        visitor.visit_string(self.try_as_single()?.to_string())
    }
    
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        let seq_access = SeqAccessor::try_from(self.try_as_map()?)?;
        visitor.visit_seq(seq_access)
    }
    
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        let seq_access = SeqAccessor::try_from(self.try_as_map()?)?;
        visitor.visit_seq(seq_access)
    }
    
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        match &self.0 {
            Value::Single(s) => {
                match s.as_str() {
                    "null" => visitor.visit_none(),
                    _ => visitor.visit_some(self)
                }
            },
            _ => visitor.visit_some(self),
        }
    }
    
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_unit()
    }
    
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        self.deserialize_unit(visitor)
    }
    
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }
    
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value> where V: serde::de::Visitor<'de> {
        let seq_access = SeqAccessor::try_from(self.try_as_map()?)?;
        visitor.visit_seq(seq_access)
    }
    
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        let seq_access = SeqAccessor::try_from(self.try_as_map()?)?;
        visitor.visit_seq(seq_access)
    }
    
    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        let seq_access = SeqAccessor::try_from(self.try_as_map()?)?;
        visitor.visit_seq(seq_access)
    }
    
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de> {
        let map_access = MapAccessor::from(self.try_as_map()?);
        visitor.visit_map(map_access)
    }
}


/// Deserialize a value from a string slice.
pub fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T> {
    let deser = FormDecoder::new(input)?;
    T::deserialize(deser)
}

/// Desrialize a value from a byte slice.
/// 
/// The byte sequence is expected to be an UTF8 encoded string.
pub fn from_bytes<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T> {
    let s = std::str::from_utf8(bytes).map_err(|_| Error::ExpectingUtf8String)?;
    from_str(s)
}

/// Desrialize a value from a byte stream.
/// 
/// The byte sequence is expected to be an UTF8 encoded string.
pub fn from_reader<T: DeserializeOwned, Reader: Read>(reader: &mut Reader) -> Result<T> {
    let mut bytes: Vec<u8> = Default::default();
    reader.read_to_end(&mut bytes).unwrap();
    from_bytes(bytes.as_slice())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{from_bytes, from_reader, from_str, tests::{ENCODED, Foo, fixture}};

    #[test]
    fn test_deserialize_str() {
        let expected = fixture();
        let got = from_str::<Foo>(ENCODED).unwrap();
        assert_eq!(got, expected);
    }

    #[test]
    fn test_deserialize_bytes() {
        let expected = fixture();
        let bytes = ENCODED.as_bytes();
        let got = from_bytes::<Foo>(bytes).unwrap();
        assert_eq!(got, expected);
    }

    #[test]
    fn test_deserialize_reader() {
        let expected = fixture();
        let bytes = ENCODED.as_bytes();
        let mut cursor = Cursor::new(bytes);
        let got = from_reader::<Foo, _>(&mut cursor).unwrap();
        assert_eq!(got, expected);
    }
}