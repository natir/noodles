mod array;

use std::io::{self, BufRead};

use byteorder::{LittleEndian, ReadBytesExt};
use noodles_sam::record::data::field::Type;

use self::array::{decode_array, Array};

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Character(u8),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Float(f32),
    String(&'a [u8]),
    Hex(&'a [u8]),
    Array(Array<'a>),
}

pub(super) fn decode_value<'a>(src: &mut &'a [u8], ty: Type) -> io::Result<Value<'a>> {
    match ty {
        Type::Character => decode_character(src),
        Type::Int8 => decode_i8(src),
        Type::UInt8 => decode_u8(src),
        Type::Int16 => decode_i16(src),
        Type::UInt16 => decode_u16(src),
        Type::Int32 => decode_i32(src),
        Type::UInt32 => decode_u32(src),
        Type::Float => decode_f32(src),
        Type::String => decode_string(src).map(Value::String),
        Type::Hex => decode_hex(src),
        Type::Array => decode_array(src).map(Value::Array),
    }
}

fn decode_character<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_u8().map(Value::Character)
}

fn decode_i8<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_i8().map(Value::Int8)
}

fn decode_u8<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_u8().map(Value::UInt8)
}

fn decode_i16<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_i16::<LittleEndian>().map(Value::Int16)
}

fn decode_u16<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_u16::<LittleEndian>().map(Value::UInt16)
}

fn decode_i32<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_i32::<LittleEndian>().map(Value::Int32)
}

fn decode_u32<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_u32::<LittleEndian>().map(Value::UInt32)
}

fn decode_f32<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    src.read_f32::<LittleEndian>().map(Value::Float)
}

fn decode_string<'a>(src: &mut &'a [u8]) -> io::Result<&'a [u8]> {
    const NUL: u8 = 0x00;

    let len = src
        .iter()
        .position(|&b| b == NUL)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "string not NUL terminated"))?;

    let buf = &src[..len];

    // +1 for the terminator.
    src.consume(len + 1);

    Ok(buf)
}

fn decode_hex<'a>(src: &mut &'a [u8]) -> io::Result<Value<'a>> {
    decode_string(src).map(Value::Hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_value() -> io::Result<()> {
        fn t(mut data: &[u8], ty: Type, expected: Value<'_>) -> io::Result<()> {
            assert_eq!(decode_value(&mut data, ty)?, expected);
            Ok(())
        }

        t(&[b'n'], Type::Character, Value::Character(b'n'))?;
        t(&[0x00], Type::Int8, Value::Int8(0))?;
        t(&[0x00], Type::UInt8, Value::UInt8(0))?;
        t(&[0x00, 0x00], Type::Int16, Value::Int16(0))?;
        t(&[0x00, 0x00], Type::UInt16, Value::UInt16(0))?;
        t(&[0x00, 0x00, 0x00, 0x00], Type::Int32, Value::Int32(0))?;
        t(&[0x00, 0x00, 0x00, 0x00], Type::UInt32, Value::UInt32(0))?;
        t(&[0x00, 0x00, 0x00, 0x00], Type::Float, Value::Float(0.0))?;
        t(
            &[b'n', b'd', b'l', b's', 0x00],
            Type::String,
            Value::String(b"ndls"),
        )?;
        t(
            &[b'C', b'A', b'F', b'E', 0x00],
            Type::Hex,
            Value::Hex(b"CAFE"),
        )?;

        t(
            &[b'C', 0x01, 0x00, 0x00, 0x00, 0x00],
            Type::Array,
            Value::Array(Array::UInt8(&[0x00])),
        )?;

        Ok(())
    }
}