use std::collections::HashMap;

use crate::lexer::{Lexer, Token};

use super::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyValue {
    pub key: String,
    pub value: String
}

impl KeyValue {
    pub fn new<Key: ToString, Value: ToString>(key: Key, value: Value) -> Self {
        Self {key: key.to_string(), value: value.to_string()}
    }
}

enum State {
    Root,
    KeyFound,
    AssignFound,
    ExpectingAmpersandOrEos,
}

pub(crate) struct Parser<'a>{
    stack: Vec<String>,
    state: State,
    lexer: Lexer<'a>
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            stack: vec![],
            state: State::Root,
            lexer: Lexer::new(input)
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<KeyValue>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let maybe_tok = self.lexer.next();

            match self.state {
                State::Root => {
                    match maybe_tok {
                        Some(Token::String(key)) => {
                            self.stack.push(key);
                            self.state = State::KeyFound;
                        },
                        None => return None,
                        _ => {
                            todo!("Expecting string token")
                        }
                    }
                },
                State::KeyFound => {
                    match maybe_tok {
                        Some(Token::Assign) => {
                            self.state = State::AssignFound;
                        }
                        _ => {
                            todo!("Expecting assign token")
                        }
                    }
                },
                State::AssignFound => {
                    match maybe_tok {
                        Some(Token::String(value)) => {
                            let key = self.stack.pop().unwrap();
                            self.state = State::ExpectingAmpersandOrEos;
                            return Some(Ok(KeyValue::new(key, value)))
                        },
                        _ => todo!("Expecting a string token")
                    }
                },
                State::ExpectingAmpersandOrEos => {
                    match maybe_tok {
                        Some(Token::Ampersand) => {
                            self.state = State::Root;
                        },
                        None => return None,
                        _ => todo!("Expecting either & or eos")
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Single(String),
    Map(Map),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Single(value)
    }
}

impl FromIterator<KeyValue> for Value {
    fn from_iter<T: IntoIterator<Item = KeyValue>>(iter: T) -> Self {
        let mut v = Value::map();
        for kv in iter.into_iter() {
            let path = kv.key.split(".").collect::<Vec<_>>();
            v.set(path.as_slice(), kv.value);
        }
        v
    }
}

impl Value {
    pub fn map() -> Self {
        Self::Map(Map::default())
    }

    pub fn set<SingleValue: ToString>(&mut self, path: &[&str], value: SingleValue) {
        if path.is_empty() {
            *self = Value::Single(value.to_string());
            return;
        }

        if self.try_as_mut_map().is_none() {
            *self = Self::Map(Map::default());
        }

        self.try_as_mut_map().unwrap().set(&path, value.to_string());

    }
    pub fn borrow(&self, path: &[&str]) -> Option<&Self> {
        if path.is_empty() {
            return Some(self)
        }

        match self {
            Value::Single(_) => None,
            Value::Map(map) => map.borrow(path),
        }
    }


    pub fn try_as_single(self) -> Option<String> {
        if let Self::Single(val) = self {
            return Some(val)
        }

        None
    }

    pub fn try_as_ref_single(&self) -> Option<&String> {
        if let Self::Single(val) = self {
            return Some(val)
        }

        None
    }

    pub fn try_as_map(self) -> Option<Map> {
         if let Self::Map(val) = self {
            return Some(val)
        }

        None       
    }

    pub fn try_as_mut_map(&mut self) -> Option<&mut Map> {
        if let Self::Map(val) = self {
            return Some(val)
        }

        None
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Map(HashMap<String, Value>);

impl IntoIterator for Map {
    type Item = (String, Value);
    type IntoIter = std::collections::hash_map::IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Map {
    pub fn take(&mut self, key: &str) -> Option<(String, Value)> {
        self.0.remove_entry(key)
    }
    
    pub fn set(&mut self, path: &[&str], value: String) {
        if path.len() == 1 {
            self.0.insert(path[0].to_string(), Value::Single(value));
            return;
        }

        let part = path[0].to_string();
        
        if self.0.contains_key(&part) {
            self.0.get_mut(&part).unwrap().set(&path[1..], value);
            return;
        }

        let mut v = Value::Map(Map::default());
        v.set(&path[1..], value);
        self.0.insert(part, v);
    }
    pub fn borrow(&self, path: &[&str]) -> Option<&Value> {
        if path.is_empty() {
            return None
        }

        let part = path[0];
        self.0.get(part).map(|v| v.borrow(&path[1..])).flatten()
    }
}

#[cfg(test)]
mod test {
    use crate::{Result, parser::{KeyValue, Value}};
    use super::Parser;

    #[test]
    fn test_parser() {
        let expected = vec![
            KeyValue::new("arg0", "false"),
            KeyValue::new("arg1", "8"),
            KeyValue::new("arg2", "9"),
            KeyValue::new("arg3", "10"),
            KeyValue::new("arg4", "11"),
            KeyValue::new("arg5", "1.01"),
            KeyValue::new("arg6", "1.02"),
            KeyValue::new("arg7", "test"),
            KeyValue::new("arg8.arg0", "nested_test"),
            KeyValue::new("arg8.arg1", "18.01"),
            KeyValue::new("arg9.0.arg0", "item0"),
            KeyValue::new("arg9.0.arg1", "20.5"),
            KeyValue::new("arg9.1.arg0", "item1"),
            KeyValue::new("arg9.1.arg1", "10.5"),     
            KeyValue::new("arg9.$length", "2")       
        ];

        let parser = Parser::new(crate::tests::ENCODED);
        let got = parser.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(got, expected);        
    }

    #[test]
    fn test_collect_nested_value() {
        let parser = Parser::new(crate::tests::ENCODED);
        
        let mut expected = Value::map();
        expected.set(&["arg0"], "false");
        expected.set(&["arg1"], "8");
        expected.set(&["arg2"], "9");
        expected.set(&["arg3"], "10");
        expected.set(&["arg4"], "11");
        expected.set(&["arg5"], "1.01");
        expected.set(&["arg6"], "1.02");
        expected.set(&["arg7"], "test");
        expected.set(&["arg8", "arg0"], "nested_test");
        expected.set(&["arg8", "arg1"], "18.01");
        expected.set(&["arg9", "0", "arg0"], "item0");
        expected.set(&["arg9", "0", "arg1"], "20.5");
        expected.set(&["arg9", "1", "arg0"], "item1");
        expected.set(&["arg9", "1", "arg1"], "10.5");
        expected.set(&["arg9", "$length"], "2");


        let got = parser.collect::<Result<Value>>().unwrap();

        assert_eq!(expected, got);
    }
}