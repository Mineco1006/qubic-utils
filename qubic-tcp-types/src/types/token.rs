use qubic_types::{QubicId, Nonce, H256};

macro_rules! impl_tokenize {
    ($impl: ty, $var: ident) => {
        impl Tokenize for $impl {
            fn tokenize(self) -> Token {
                Token::$var(self)
            }
        }
    };
}

pub trait Tokenize {
    fn tokenize(self) -> Token;
}

pub enum Token {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    Id(QubicId),
    H256(H256)
}

impl_tokenize!(u8, U8);
impl_tokenize!(u16, U16);
impl_tokenize!(u32, U32);
impl_tokenize!(u64, U64);
impl_tokenize!(i8, I8);
impl_tokenize!(i16, I16);
impl_tokenize!(i32, I32);
impl_tokenize!(i64, I64);
impl_tokenize!(QubicId, Id);
impl_tokenize!(H256, H256);