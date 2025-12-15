use std::io::Write;

use serde::Serialize;

use crate::Error;

use super::Result;

#[derive(Default)]
pub struct Map(Vec<(String, Value)>);

pub enum Value {
    Map(Map),
    Seq(Vec<Value>),
    Single(String)
}

impl Value {
    pub fn to_string(self) -> String {
        let mut output = String::default();
        self.write(Default::default(), &mut output);
        if output.ends_with("&") {
            output.pop();
        }
        output
    }

    fn write(self, path: String, output: &mut String) {
        let prefix = if path.len() > 0 { format!("{path}.") } else { Default::default() };

        match self {
            Value::Map(map) => {
                map.0.into_iter()
                    .for_each(|(k, v)| {
                        let cpth = format!("{}{k}", prefix);
                        v.write(cpth, output);
                    });
            },
            Value::Seq(values) => {
                let len = values.len();
                values.into_iter().enumerate().for_each(|(i, v)| {
                    let cpth =  format!("{}{i}", prefix);
                    v.write(cpth, output);
                });
                *output += &format!("{}$length={len}&", prefix);
            },
            Value::Single(value) => {
                *output += &format!("{}={value}&", path);
            },
        }
    }
}

impl Value {
    pub fn as_single(self) -> String {
        self.try_as_single().unwrap()
    }

    pub fn try_as_single(self) -> Option<String> {
        match self {
            Value::Single(value) => Some(value),
            _ => None
        }
    }

    pub fn try_as_mut_seq(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Seq(value) => Some(value),
            _ => None
        }       
    }

    pub fn as_mut_seq(&mut self) -> &mut Vec<Value> {
        self.try_as_mut_seq().unwrap()
    }

    pub fn try_as_map(self) -> Option<Map> {
        match self {
            Value::Map(map) => Some(map),
            _ => None
        }
    }

    pub fn as_map(self) -> Map {
        self.try_as_map().unwrap()
    }

    pub fn try_as_mut_map(&mut self) -> Option<&mut Map> {
        match self {
            Value::Map(value) => Some(value),
            _ => None
        }       
    }
    
    pub fn as_mut_map(&mut self) -> &mut Map {
        self.try_as_mut_map().unwrap()
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Single(value)
    }
}

#[derive(Default)]
pub struct FormEncoder(Vec<Value>);

impl FormEncoder {
    pub fn push<V: Into<Value>>(&mut self, item: V) {
        self.0.push(item.into());
    }

    pub fn pop_key_value(&mut self) -> Result<()> {
        let value = self.0.pop().unwrap();
        let key = self.0.pop().unwrap().as_single();
        
        self.0.last_mut()
            .unwrap()
            .as_mut_map()
            .0
            .push((key, value));

        Ok(())
    }

    pub fn pop_element(&mut self) -> Result<()> {
        let value = self.0.pop().unwrap();
        self.0.last_mut().unwrap().as_mut_seq().push(value);
        Ok(())
    }
}

impl serde::ser::SerializeMap for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize {
        let k = key.serialize(FormEncoder::default())?;
        self.push(k);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize {
        let v = value.serialize(FormEncoder::default())?;
        self.push(v);
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + serde::Serialize,
        V: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)?;
        self.pop_key_value()?;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::ser::SerializeStruct for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize {
            let v = value.serialize(FormEncoder::default())?;
            
            self.push(key.to_string());
            self.push(v);

            self.pop_key_value()?;
            
            Ok(())
        
    }

    fn end(mut self) -> Result<Self::Ok> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::ser::SerializeSeq for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize {
        let element: Value = value.serialize(FormEncoder::default())?;
        self.0.last_mut().unwrap().as_mut_seq().push(element);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::ser::SerializeTuple for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize {
        let element: Value = value.serialize(FormEncoder::default())?;
        self.0.last_mut().unwrap().as_mut_seq().push(element);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::ser::SerializeTupleStruct for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize {
            let v = value.serialize(FormEncoder::default())?;            
            self.push(v);
            self.pop_element()?;            
            Ok(())
    }

    fn end(mut self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::ser::SerializeTupleVariant for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize {
        let v = value.serialize(FormEncoder::default())?;            
        self.push(v);
        self.pop_element()?;   
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::ser::SerializeStructVariant for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize {
        let v = value.serialize(FormEncoder::default())?;       
        self.push(Value::Single(key.to_string()));
        self.push(v);
        self.pop_key_value()?;   
        Ok(())
    }

    fn end(mut self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(self.0.pop().unwrap())
    }
}

impl serde::Serializer for FormEncoder {
    type Ok = Value;
    type Error = super::Error;

    type SerializeStruct = Self;
    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;   
    type SerializeStructVariant = Self;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok> {
        let value= match v {
            true => "true",
            false => "false",
        };

        self.push(value.to_string());

        Ok(value.to_string().into())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(v.to_string().into())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let value = v.to_string().replace("\"", "\\\"");
        Ok(format!("\"{value}\"").into())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let vec = v
            .iter()
            .map(|v| v.serialize(FormEncoder::default()))
            .collect::<Result<Vec<_>>>()?;

        Ok(Value::Seq(vec))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Value::Single("null".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize {
        value.serialize(self)
    }

    fn serialize_seq(mut self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.0.push(Value::Seq(Default::default()));
        Ok(self)
    }

    fn serialize_tuple(mut self, _len: usize) -> Result<Self::SerializeTuple> {
        self.0.push(Value::Seq(Default::default()));
        Ok(self)
    }

    fn serialize_tuple_struct(mut self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        self.0.push(Value::Seq(Default::default()));
        Ok(self)
    }

    fn serialize_tuple_variant(mut self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize,) -> Result<Self::SerializeTupleVariant> {
        self.0.push(Value::Seq(Default::default()));
        Ok(self)
    }

    fn serialize_map(mut self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.push(Value::Map(Default::default()));
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize,) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(mut self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize,) -> Result<Self::SerializeStructVariant> {
        self.0.push(Value::Map(Default::default()));
        Ok(self)
    }
}

/// Serialize the value
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    value.serialize(FormEncoder::default()).map(|v| v.to_string())
}

/// Serialize and write the value into a byte stream.
pub fn to_writer<T: Serialize, Writer: Write>(value: &T, writer: &mut Writer) -> Result<()> {
    let str = to_string(value)?;
    writer.write_all(str.as_bytes()).map_err(|err| Error::IoError(err.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{ser::to_writer, tests::{ENCODED, fixture}, to_string};

    #[test]
    fn test_serialize_to_string() {
        let expected = fixture();
        let got = to_string(&expected).unwrap();
        assert_eq!(ENCODED, got);
    }

    #[test]
    fn test_serialize_to_writer() {
        let expected = fixture();
        let mut got: Vec<u8> = Default::default();
        to_writer(&expected, &mut got).unwrap();
        assert_eq!(
            ENCODED, 
            std::str::from_utf8(got.as_slice()).unwrap()
        );
    }
}