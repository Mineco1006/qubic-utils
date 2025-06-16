use qubic_types::{QubicId, H256};

macro_rules! impl_tokenize {
    ($impl: ty, $var: ident) => {
        impl Tokenize for $impl {
            fn tokenize(self) -> Token {
                Token::$var(self)
            }
        }
    };
}

macro_rules! impl_into_tokens {
    ($($idx:tt $t: tt), +) => {
        impl<$($t, )+> IntoTokens for ($($t, )+) where $($t: Tokenize,)+ {
            fn into_tokens(self) -> Vec<Token> {
                let mut tokens = Vec::new();

                ($(
                    tokens.push((self.$idx).tokenize()),
                )+);

                tokens
            }
        }
    };

    ($len: expr) => {
        impl<T: Tokenize> IntoTokens for [T; $len] {
            fn into_tokens(self) -> Vec<Token> {
                let mut tokens = Vec::with_capacity($len);

                for i in self {
                    tokens.push(i.tokenize());
                }

                tokens
            }
        }
    }
}

impl<T> IntoTokens for T where T: Tokenize {
    fn into_tokens(self) -> Vec<Token> {
        vec![self.tokenize()]
    }
}

impl IntoTokens for Vec<Token> {
    fn into_tokens(self) -> Vec<Token> {
        self
    }
}

impl_into_tokens!(0 A);
impl_into_tokens!(0 A, 1 B);
impl_into_tokens!(0 A, 1 B, 2 C);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O);
impl_into_tokens!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P);
impl_into_tokens!(1);
impl_into_tokens!(2);
impl_into_tokens!(3);
impl_into_tokens!(4);
impl_into_tokens!(5);
impl_into_tokens!(6);
impl_into_tokens!(7);
impl_into_tokens!(8);


pub trait Tokenize {
    fn tokenize(self) -> Token;
}

pub trait IntoTokens {
    fn into_tokens(self) -> Vec<Token>;
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