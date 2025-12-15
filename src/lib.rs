/// # Serde www-form-urlencoded
/// 
/// This crate implements *ser*ialization and *de*serialization of www-form-urlencoded data.
/// 
/// # Format
/// Map or struct data are flat-encoded. 
/// 
/// Exemple
/// ```
/// use serde::{Serialize};
/// 
/// #[derive(Serialize)]
/// struct Foo<'a> {
///   foo0: Bar<'a>,
///   foo1: u8
/// }
/// 
/// #[derive(Serialize)]  
/// struct Bar<'a> {
///   bar0: bool,
///   bar1: &'a str
/// }
/// 
/// const ITEM: Foo<'static> = Foo {
///   foo0: Bar {
///     bar0: true,
///     bar1: "test"
///   },
///   foo1: 2
/// };
/// 
/// assert_eq!(
///     &serde_www_form_urlencoded::to_string(&ITEM).unwrap(),
///     "foo0.bar0=true&foo0.bar1=\"test\"&foo1=2"
/// );
/// ```
/// 
/// Sequence are flat-encoded with a $length attribute to keep record of the number of items.
/// 
/// ```
/// use serde::{Serialize};
/// 
/// #[derive(Serialize)]
/// struct Foo<'a> {
///   foo0: &'a[u8]
/// }
/// 
/// const ITEM: Foo<'static> = Foo {foo0: &[0,1,2,3,4]};
/// 
/// assert_eq!(
///     &serde_www_form_urlencoded::to_string(&ITEM).unwrap(),
///     "foo0.0=0&foo0.1=1&foo0.2=2&foo0.3=3&foo0.4=4&foo0.$length=5"
/// );
/// ```

mod error;
mod parser;
mod lexer;
mod de;
mod ser;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub use ser::{FormEncoder as Serializer, to_string, to_writer};
pub use de::{from_str, from_bytes, from_reader, FormDecoder as Deserializer};

#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct Foo {
        arg0: bool,
        arg1: u8,
        arg2: u16,
        arg3: u32,
        arg4: u64,
        arg5: f32,
        arg6: f64,
        arg7: String,
        arg8: Nested,
        arg9: Vec<Nested>
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct Nested {
        arg0: String,
        arg1: f32
    }
    
    pub fn fixture() -> Foo {
        Foo {
            arg0: false,
            arg1: 8,
            arg2: 9,
            arg3: 10,
            arg4: 11,
            arg5: 1.01,
            arg6: 1.02,
            arg7: "test".to_string(),
            arg8: Nested { arg0: "nested_test".to_string(), arg1: 18.01 },
            arg9: vec![
                Nested { arg0: "item0".to_string(), arg1: 20.5 },
                Nested { arg0: "item1".to_string(), arg1: 10.5 },
            ]
        }
    }

    pub const ENCODED: &'static str = "arg0=false&arg1=8&arg2=9&arg3=10&arg4=11&arg5=1.01&arg6=1.02&arg7=\"test\"&arg8.arg0=\"nested_test\"&arg8.arg1=18.01&arg9.0.arg0=\"item0\"&arg9.0.arg1=20.5&arg9.1.arg0=\"item1\"&arg9.1.arg1=10.5&arg9.$length=2";
    
}