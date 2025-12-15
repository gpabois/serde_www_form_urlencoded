use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Error {
    ExpectingI8,
    ExpectingI16,
    ExpectingI32,
    ExpectingI64,
    ExpectingU8,
    ExpectingU16,
    ExpectingU32,
    ExpectingU64,
    ExpectingUsize,
    ExpectingF32,
    ExpectingF64,  
    ExpectingChar,
    ExpectingBool,
    ExpectingMap,
    ExpectingString,
    ExpectingUtf8String,
    MissingSequenceLength,
    MissingSequenceItem,
    MissingMapValue,
    IoError(String),
    Custom(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExpectingI8 => f.write_str("expecting i8"),
            Error::ExpectingI16 => f.write_str("expecting i16"),
            Error::ExpectingI32 => f.write_str("expecting i32"),
            Error::ExpectingI64 => f.write_str("expecting i64"),
            Error::ExpectingU8 => f.write_str("expecting u8"),
            Error::ExpectingU16 => f.write_str("expecting u16"),
            Error::ExpectingU32 => f.write_str("expecting u32"),
            Error::ExpectingU64 => f.write_str("expecting u64"),
            Error::ExpectingUsize => f.write_str("expecting usize"),
            Error::ExpectingF32 => f.write_str("expecting f32"),
            Error::ExpectingF64 => f.write_str("expecting f64"),
            Error::ExpectingChar => f.write_str("expecting char"),
            Error::ExpectingBool => f.write_str("expecting bool"),
            Error::ExpectingMap => f.write_str("expecting map"),
            Error::ExpectingString => f.write_str("expecting string"),
            Error::MissingSequenceLength => f.write_str("expecting $length"),
            Error::MissingSequenceItem => f.write_str("expecting sequence item"),
            Error::MissingMapValue => f.write_str("expecting map value"),
            Error::Custom(custom) => custom.fmt(f),
            Error::ExpectingUtf8String => f.write_str("expecting bytes sequence to be an encoded utf-8 string"),
            Error::IoError(msg) => write!(f, "IO error : {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self where T: Display {
        Self::Custom(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self where T:Display {
        Self::Custom(msg.to_string())
    }
}