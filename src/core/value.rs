use crate::util::slice_to_u64_be;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Value {
    Unit,
    SumL(Box<Value>),
    SumR(Box<Value>),
    Prod(Box<Value>, Box<Value>),
}

impl Value {
    #![allow(clippy::len_without_is_empty)]
    /// The length, in bits, of the value when encoded in the Bit Machine
    pub fn len(&self) -> usize {
        match *self {
            Value::Unit => 0,
            Value::SumL(ref s) => 1 + s.len(),
            Value::SumR(ref s) => 1 + s.len(),
            Value::Prod(ref s, ref t) => s.len() + t.len(),
        }
    }

    /// Encode a single bit as a value. Will panic if the input is out of range
    pub fn u1(n: u8) -> Value {
        match n {
            0 => Value::SumL(Box::new(Value::Unit)),
            1 => Value::SumR(Box::new(Value::Unit)),
            x => panic!("{} out of range for Value::u1", x),
        }
    }

    /// Encode a two-bit number as a value. Will panic if the input is out of range
    pub fn u2(n: u8) -> Value {
        let b0 = (n & 2) / 2;
        let b1 = n & 1;
        if n > 3 {
            panic!("{} out of range for Value::u2", n);
        }
        Value::Prod(Box::new(Value::u1(b0)), Box::new(Value::u1(b1)))
    }

    /// Encode a four-bit number as a value. Will panic if the input is out of range
    pub fn u4(n: u8) -> Value {
        let w0 = (n & 12) / 4;
        let w1 = n & 3;
        if n > 15 {
            panic!("{} out of range for Value::u4", n);
        }
        Value::Prod(Box::new(Value::u2(w0)), Box::new(Value::u2(w1)))
    }

    /// Encode an eight-bit number as a value
    pub fn u8(n: u8) -> Value {
        let w0 = n >> 4;
        let w1 = n & 0xf;
        Value::Prod(Box::new(Value::u4(w0)), Box::new(Value::u4(w1)))
    }

    /// Encode a 16-bit number as a value
    pub fn u16(n: u16) -> Value {
        let w0 = (n >> 8) as u8;
        let w1 = (n & 0xff) as u8;
        Value::Prod(Box::new(Value::u8(w0)), Box::new(Value::u8(w1)))
    }

    /// Encode a 32-bit number as a value
    pub fn u32(n: u32) -> Value {
        let w0 = (n >> 16) as u16;
        let w1 = (n & 0xffff) as u16;
        Value::Prod(Box::new(Value::u16(w0)), Box::new(Value::u16(w1)))
    }

    /// Encode a 32-bit number as a value
    pub fn u64(n: u64) -> Value {
        let w0 = (n >> 32) as u32;
        let w1 = (n & 0xffff_ffff) as u32;
        Value::Prod(Box::new(Value::u32(w0)), Box::new(Value::u32(w1)))
    }

    /// Encode a 32 byte number into value. Useful for encoding 32 pubkeys/hashes
    pub fn u256_from_slice(v: &[u8]) -> Value {
        assert!(v.len() == 32);
        Value::Prod(
            Box::new(Value::Prod(
                Box::new(Value::u64(slice_to_u64_be(&v[0..8]))),
                Box::new(Value::u64(slice_to_u64_be(&v[8..16]))),
            )),
            Box::new(Value::Prod(
                Box::new(Value::u64(slice_to_u64_be(&v[16..24]))),
                Box::new(Value::u64(slice_to_u64_be(&v[24..32]))),
            )),
        )
    }

    /// Encode a 64(pair(32, 32)) byte number into value.
    /// Useful for encoding 64 byte signatures
    pub fn u512_from_slice(v: &[u8]) -> Value {
        assert!(v.len() == 64);
        Value::Prod(
            Box::new(Value::u256_from_slice(&v[0..32])),
            Box::new(Value::u256_from_slice(&v[32..64])),
        )
    }

    /// Encode value as big-endian byte string.
    /// Fails if underlying bit string has length not divisible by 8
    pub fn try_to_bytes(&self) -> Result<Vec<u8>, &str> {
        let (bytes, bit_length) = self.to_bytes_len();

        if bit_length % 8 == 0 {
            Ok(bytes)
        } else {
            Err("Length of bit string that encodes this value is not divisible by 8!")
        }
    }

    /// Encode value as big-endian byte string.
    /// Trailing zeroes are added as padding if underlying bit string has length not divisible by 8.
    /// The length of said bit string is returned as second argument
    pub fn to_bytes_len(&self) -> (Vec<u8>, usize) {
        let mut bytes = vec![];
        let mut unfinished_byte = Vec::with_capacity(8);
        let mut value_stack = vec![self];

        while let Some(value) = value_stack.pop() {
            match value {
                Value::Unit => {}
                Value::SumL(l) => {
                    unfinished_byte.push(false);
                    value_stack.push(l);
                }
                Value::SumR(r) => {
                    unfinished_byte.push(true);
                    value_stack.push(r);
                }
                Value::Prod(l, r) => {
                    value_stack.push(r);
                    value_stack.push(l);
                }
            }

            if unfinished_byte.len() == 8 {
                bytes.push(unfinished_byte.iter().fold(0, |acc, &b| acc * 2 + b as u8));
                unfinished_byte.clear();
            }
        }

        let bit_length = bytes.len() * 8 + unfinished_byte.len();

        if !unfinished_byte.is_empty() {
            unfinished_byte.resize(8, false);
            bytes.push(unfinished_byte.iter().fold(0, |acc, &b| acc * 2 + b as u8));
        }

        (bytes, bit_length)
    }

    /// Convenience constructor for a left sum of a value
    pub fn sum_l(a: Value) -> Value {
        Value::SumL(Box::new(a))
    }

    /// Convenience constructor for a right sum of a value
    pub fn sum_r(a: Value) -> Value {
        Value::SumR(Box::new(a))
    }

    /// Convenience constructor for product of two values
    pub fn prod(a: Value, b: Value) -> Value {
        Value::Prod(Box::new(a), Box::new(b))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Unit => f.write_str("ε"),
            Value::SumL(ref sub) => {
                f.write_str("0")?;
                if **sub != Value::Unit {
                    write!(f, "{}", sub)
                } else {
                    Ok(())
                }
            }
            Value::SumR(ref sub) => {
                f.write_str("1")?;
                if **sub != Value::Unit {
                    write!(f, "{}", sub)
                } else {
                    Ok(())
                }
            }
            Value::Prod(ref l, ref r) => {
                write!(f, "({},", l)?;
                write!(f, "{})", r)
            }
        }
    }
}