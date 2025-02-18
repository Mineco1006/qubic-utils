#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_addcarry_u64, _subborrow_u64};

use alloc::format;

use base64::Engine;
use core::{
    fmt::{Debug, Display},
    ptr::copy_nonoverlapping,
    str::FromStr,
};

use four_q::{
    consts::{CURVE_ORDER_0, CURVE_ORDER_1, CURVE_ORDER_2, CURVE_ORDER_3, MONTGOMERY_R_PRIME, ONE},
    ops::{decode, ecc_mul, ecc_mul_double, ecc_mul_fixed, encode, montgomery_multiply_mod_order},
    types::PointAffine,
};
use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};

use crate::qubic_types::{
    errors::QubicError, traits::ToBytes, MiningSeed, Nonce, QubicId, QubicTxHash, QubicWallet,
    Signature,
};

fn addcarry_u64(c_in: u8, a: u64, b: u64, out: &mut u64) -> u8 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        _addcarry_u64(c_in, a, b, out)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        let c_out = a.overflowing_add(b);
        let c_out1 = c_out.0.overflowing_add(if c_in != 0 { 1 } else { 0 });

        *out = c_out1.0;

        (c_out.1 || c_out1.1) as u8
    }
}

fn subborrow_u64(b_in: u8, a: u64, b: u64, out: &mut u64) -> u8 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        _subborrow_u64(b_in, a, b, out)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        let b_out = a.overflowing_sub(b);
        let b_out1 = b_out.0.overflowing_sub(if b_in != 0 { 1 } else { 0 });

        *out = b_out1.0;

        (b_out.1 || b_out1.1) as u8
    }
}

impl FromStr for QubicId {
    type Err = QubicError;

    #[inline]
    fn from_str(id: &str) -> Result<Self, Self::Err> {
        let mut buffer = [0u8; 32];

        if !id
            .chars()
            .all(|c| c.is_uppercase() && c.is_ascii_alphabetic())
        {
            return Err(QubicError::InvalidIdFormatError { ident: "ID" });
        }

        let id = id.as_bytes();

        if id.len() != 60 {
            return Err(QubicError::InvalidIdLengthError {
                ident: "ID",
                expected: 60,
                found: id.len(),
            });
        }

        for i in 0..4 {
            for j in (0..14usize).rev() {
                let im = u64::from_le_bytes(buffer[i << 3..(i << 3) + 8].try_into().unwrap()) * 26
                    + (id[i * 14 + j] - b'A') as u64;
                let im = im.to_le_bytes();

                for k in 0..8 {
                    buffer[(i << 3) + k] = im[k];
                }
            }
        }

        Ok(Self(buffer))
    }
}

impl QubicId {
    #[inline]
    pub fn check_id(id: &str) -> Result<(), QubicError> {
        if !id
            .chars()
            .all(|c| c.is_uppercase() && c.is_ascii_alphabetic())
        {
            return Err(QubicError::InvalidIdFormatError { ident: "ID" });
        }

        let id = id.as_bytes();

        if id.len() != 60 {
            return Err(QubicError::InvalidIdLengthError {
                ident: "ID",
                expected: 55,
                found: id.len(),
            });
        }

        Ok(())
    }

    #[inline]
    pub fn get_identity(&self) -> String {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment =
                u64::from_le_bytes(self.0[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + b'A';
                public_key_fragment /= 26;
            }
        }

        let mut identity_bytes_checksum = [0u8; 3];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&self.0);
        kg.into_xof().squeeze(&mut identity_bytes_checksum);
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u64
            | (identity_bytes_checksum[1] as u64) << 8
            | (identity_bytes_checksum[2] as u64) << 16;
        identity_bytes_checksum &= 0x3FFFF;
        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'A';
            identity_bytes_checksum /= 26;
        }

        String::from_utf8(identity.to_vec()).unwrap()
    }

    #[inline]
    pub fn get_identity_bytes(&self) -> [u8; 60] {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment =
                u64::from_le_bytes(self.0[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + b'A';
                public_key_fragment /= 26;
            }
        }

        let mut identity_bytes_checksum = [0u8; 3];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&self.0);
        kg.into_xof().squeeze(&mut identity_bytes_checksum);
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u64
            | (identity_bytes_checksum[1] as u64) << 8
            | (identity_bytes_checksum[2] as u64) << 16;
        identity_bytes_checksum &= 0x3FFFF;
        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'A';
            identity_bytes_checksum /= 26;
        }

        identity
    }

    #[inline]
    pub fn from_slice(slice: &[u8]) -> Result<Self, QubicError> {
        if let Ok(arr) = slice.try_into() {
            Ok(Self(arr))
        } else {
            Err(QubicError::InvalidIdLengthError {
                ident: "PUBLIC_KEY",
                expected: 32,
                found: slice.len(),
            })
        }
    }

    #[inline]
    pub fn from_le_u64(le_u64: [u64; 4]) -> Self {
        Self(core::array::from_fn(|i| le_u64[i / 8].to_le_bytes()[i % 8]))
    }

    #[inline]
    pub fn from_be_u64(be_u64: [u64; 4]) -> Self {
        Self(core::array::from_fn(|i| be_u64[i / 8].to_be_bytes()[i % 8]))
    }

    #[inline]
    pub fn to_le_u64(self) -> [u64; 4] {
        let mut ret = [[0; 8]; 4];

        for i in 0..32 {
            ret[i / 8][i % 8] = self.0[i];
        }

        core::array::from_fn(|i| u64::from_le_bytes(ret[i]))
    }

    #[inline]
    pub fn to_be_u64(self) -> [u64; 4] {
        let mut ret = [[0; 8]; 4];

        for i in 0..32 {
            ret[i / 8][i % 8] = self.0[i];
        }

        core::array::from_fn(|i| u64::from_be_bytes(ret[i]))
    }

    pub fn from_contract_id(contract_id: u32) -> QubicId {
        QubicId::from_le_u64([contract_id as u64, 0, 0, 0])
    }

    /// Verifies signature from message
    ///
    /// ```
    /// use qubic_rs::qubic_types::QubicId;
    ///
    /// const SIGNATURE: Signature = Signature([0; 64]);
    ///
    /// let message = "qubic".as_bytes();
    ///
    /// let id = QubicId::from_str("BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK");
    ///
    /// id.verify(message, SIGNATURE);
    /// ```
    #[inline]
    pub fn verify<T: ToBytes>(&self, message: T, signature: Signature) -> bool {
        let mut digest: [u8; 32] = [0; 32];
        let msg_bytes = message.to_bytes();
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&msg_bytes);
        kg.into_xof().squeeze(&mut digest);

        self.verify_raw(digest, signature)
    }

    /// Verifies signature from digest
    ///
    /// ```
    /// use qubic_rs::qubic_types::QubicId;
    ///
    /// const SIGNATURE: Signature = Signature([0; 64]);
    ///
    /// let digest = [0; 32]; // use KangarooTwelve to generate digest of your message data
    ///
    /// let id = QubicId::from_str("BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK");
    ///
    /// id.verify_raw(digest, SIGNATURE);
    /// ```
    #[inline]
    pub fn verify_raw(&self, message_digest: [u8; 32], signature: Signature) -> bool {
        let signature = signature.0;
        let public_key = self.0;
        if public_key[15] & 0x80 != 0
            || signature[15] & 0x80 != 0
            || signature[62] & 0xC0 != 0
            || signature[63] != 0
        {
            return false;
        }

        let mut a = PointAffine::default();
        let (mut temp, mut h) = ([0u8; 96], [0u8; 64]);

        if !decode(&public_key, &mut a) {
            return false;
        }

        unsafe {
            copy_nonoverlapping(signature.as_ptr(), temp.as_mut_ptr(), 32);
            copy_nonoverlapping(public_key.as_ptr(), temp.as_mut_ptr().offset(32), 32);
            copy_nonoverlapping(message_digest.as_ptr(), temp.as_mut_ptr().offset(64), 32);
        }

        let mut sig: [u64; 8] = signature
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let mut kg = KangarooTwelve::new(b"");
        kg.update(&temp);
        kg.into_xof().squeeze(&mut h);

        let mut h: [u64; 8] = h
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        if !ecc_mul_double(&mut sig[4..], &mut h, &mut a) {
            return false;
        }

        let mut a_bytes = [0u8; 64];

        unsafe {
            copy_nonoverlapping(
                &a as *const PointAffine as *mut u8,
                a_bytes.as_mut_ptr(),
                64,
            );
        }

        encode(&mut a, &mut a_bytes);

        signature[0..32] == a_bytes[0..32]
    }
}

impl Debug for QubicId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let id = self.get_identity();
        f.write_str(&format!("{}...{}", &id[..5], &id[55..]))
    }
}

impl Display for QubicId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.get_identity())
    }
}

impl QubicWallet {
    /// Generates a wallet from the given input seed
    ///
    /// ```
    /// use qubic_rs::qubic_types::QubicWallet;
    /// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    /// ```
    pub fn from_seed(seed: &str) -> Result<Self, QubicError> {
        let subseed = Self::get_subseed(seed)?;
        let private_key = Self::get_private_key(&subseed);
        let public_key = Self::get_public_key(&private_key);

        Ok(Self {
            private_key,
            public_key: QubicId(public_key),
            subseed,
        })
    }

    pub fn get_subseed(seed: &str) -> Result<[u8; 32], QubicError> {
        if !seed
            .chars()
            .all(|c| c.is_lowercase() && c.is_ascii_alphabetic())
        {
            return Err(QubicError::InvalidIdFormatError { ident: "SEED" });
        }

        if seed.len() != 55 {
            return Err(QubicError::InvalidIdLengthError {
                ident: "SEED",
                expected: 55,
                found: seed.len(),
            });
        }

        let seed = seed.as_bytes();
        let mut seed_bytes = [0u8; 55];

        for i in 0..55 {
            seed_bytes[i] = seed[i] - b'a';
        }

        let mut subseed = [0u8; 32];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&seed_bytes);
        kg.into_xof().squeeze(&mut subseed);

        Ok(subseed)
    }

    // Generates the private key from input subseed
    #[inline(always)]
    pub fn get_private_key(subseed: &[u8; 32]) -> [u8; 32] {
        let mut pk = [0u8; 32];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(subseed);
        kg.into_xof().squeeze(&mut pk);

        pk
    }

    /// SchnorrQ public key generation
    #[inline(always)]
    pub fn get_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        let mut p = PointAffine::default();
        let private_key = private_key
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>();

        ecc_mul_fixed(&private_key, &mut p);
        let mut private_key: [u8; 32] = private_key
            .into_iter()
            .flat_map(u64::to_le_bytes)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        encode(&mut p, &mut private_key);

        private_key
    }

    /// Get the identity of the wallet
    ///
    /// ```
    /// use qubic_rs::qubic_types::QubicWallet;
    /// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    ///
    /// assert_eq(wallet.get_identity(), "BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK");
    /// ```
    #[inline(always)]
    pub fn get_identity(&self) -> String {
        self.public_key.get_identity()
    }

    pub fn get_shared_key(&self) -> Result<[u8; 32], QubicError> {
        let mut a = PointAffine::default();

        if self.public_key.0[15] & 0x80 != 0 {
            return Err(QubicError::FormattingError);
        }

        if !decode(&self.public_key.0, &mut a) {
            return Err(QubicError::EllipticCurveError);
        }

        let private_key_u64 = self
            .private_key
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>();

        unsafe {
            if !ecc_mul(&mut *(&mut a as *mut PointAffine), &private_key_u64, &mut a) {
                return Err(QubicError::EllipticCurveError);
            }
        }

        if a.x[0][0] == 0
            && a.x[1][0] == 0
            && a.x[0][1] == 0
            && a.x[1][1] == 0
            && a.y[0][0] == 0
            && a.y[1][0] == 0
            && a.y[0][1] == 0
            && a.y[1][1] == 0
        {
            return Err(QubicError::EllipticCurveError);
        }

        let mut shared_key = [0u8; 32];

        unsafe {
            copy_nonoverlapping(a.y.as_ptr() as *mut u8, shared_key.as_mut_ptr(), 32);
        }

        Ok(shared_key)
    }

    /// SchnorrQ signature generation from message
    ///
    /// ```
    /// use qubic_rs::qubic_types::QubicWallet;
    /// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    ///
    /// let message = "qubic".as_bytes();
    ///
    /// let signature = wallet.sign(message);
    /// ```
    pub fn sign<T: ToBytes>(&self, message: T) -> Signature {
        let mut message_digest = [0; 32];
        let msg_bytes = message.to_bytes();
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&msg_bytes);
        kg.into_xof().squeeze(&mut message_digest);

        self.sign_raw(message_digest)
    }

    /// SchnorrQ signature generation from message digest
    ///
    /// ```
    /// use qubic_rs::qubic_types::QubicWallet;
    /// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    ///
    /// let digest = [0; 32]; // use KangarooTwelve to generate digest of your message data
    ///
    /// let signature = wallet.sign_raw(digest);
    /// ```
    pub fn sign_raw(&self, message_digest: [u8; 32]) -> Signature {
        let mut r_a = PointAffine::default();
        let (mut k, mut h, mut temp) = ([0u8; 64], [0u8; 64], [0u8; 96]);
        let mut r = [0u8; 64];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&self.subseed);
        kg.into_xof().squeeze(&mut k);

        let mut signature = [0u8; 64];

        unsafe {
            copy_nonoverlapping(k.as_ptr().offset(32), temp.as_mut_ptr().offset(32), 32);
            copy_nonoverlapping(message_digest.as_ptr(), temp.as_mut_ptr().offset(64), 32);

            let mut kg = KangarooTwelve::new(b"");
            kg.update(&temp[32..]);
            let mut im = [0u8; 64];
            kg.into_xof().squeeze(&mut im);

            copy_nonoverlapping(im.as_ptr(), r.as_mut_ptr(), 64);
            let k: [u64; 8] = k
                .chunks_exact(8)
                .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let mut r: [u64; 8] = r
                .chunks_exact(8)
                .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            ecc_mul_fixed(&r, &mut r_a);

            encode(&mut r_a, &mut signature);
            let mut signature_i: [u64; 8] = signature
                .chunks_exact(8)
                .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            copy_nonoverlapping(signature_i.as_ptr() as *mut u8, temp.as_mut_ptr(), 32);
            copy_nonoverlapping(self.public_key.0.as_ptr(), temp.as_mut_ptr().offset(32), 32);

            let mut kg = KangarooTwelve::new(b"");
            kg.update(&temp);
            kg.into_xof().squeeze(&mut h);

            let mut h: [u64; 8] = h
                .chunks_exact(8)
                .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let r_i = r;
            montgomery_multiply_mod_order(&r_i, &MONTGOMERY_R_PRIME, &mut r);
            let r_i = r;
            montgomery_multiply_mod_order(&r_i, &ONE, &mut r);
            let h_i = h;
            montgomery_multiply_mod_order(&h_i, &MONTGOMERY_R_PRIME, &mut h);
            let h_i = h;
            montgomery_multiply_mod_order(&h_i, &ONE, &mut h);
            montgomery_multiply_mod_order(&k, &MONTGOMERY_R_PRIME, &mut signature_i[4..]);
            let h_i = h;
            montgomery_multiply_mod_order(&h_i, &MONTGOMERY_R_PRIME, &mut h);
            let mut s_i = [0u64; 4];
            s_i.copy_from_slice(&signature_i[4..]);
            montgomery_multiply_mod_order(&s_i, &h, &mut signature_i[4..]);
            s_i.copy_from_slice(&signature_i[4..]);
            montgomery_multiply_mod_order(&s_i, &ONE, &mut signature_i[4..]);

            if subborrow_u64(
                subborrow_u64(
                    subborrow_u64(
                        subborrow_u64(0, r[0], signature_i[4], &mut signature_i[4]),
                        r[1],
                        signature_i[5],
                        &mut signature_i[5],
                    ),
                    r[2],
                    signature_i[6],
                    &mut signature_i[6],
                ),
                r[3],
                signature_i[7],
                &mut signature_i[7],
            ) != 0
            {
                addcarry_u64(
                    addcarry_u64(
                        addcarry_u64(
                            addcarry_u64(0, signature_i[4], CURVE_ORDER_0, &mut signature_i[4]),
                            signature_i[5],
                            CURVE_ORDER_1,
                            &mut signature_i[5],
                        ),
                        signature_i[6],
                        CURVE_ORDER_2,
                        &mut signature_i[6],
                    ),
                    signature_i[7],
                    CURVE_ORDER_3,
                    &mut signature_i[7],
                );
            }

            signature = signature_i
                .into_iter()
                .flat_map(u64::to_le_bytes)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
        }

        Signature(signature)
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut hex_slice = [0; 128];
        hex::encode_to_slice(self.0, &mut hex_slice).unwrap();
        f.write_str(&format!(
            "0x{}",
            String::from_utf8(hex_slice.to_vec()).unwrap()
        ))
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut hex_slice = [0; 128];
        hex::encode_to_slice(self.0, &mut hex_slice).unwrap();
        f.write_str(&format!(
            "{}",
            String::from_utf8(hex_slice.to_vec()).unwrap()
        ))
    }
}

impl Debug for MiningSeed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&format!("{}", self.get_identity()))
    }
}

impl Display for MiningSeed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&format!("{}", self.get_identity()))
    }
}

impl MiningSeed {
    #[inline]
    pub fn from_le_u64(le_u64: [u64; 4]) -> Self {
        Self(core::array::from_fn(|i| le_u64[i / 8].to_le_bytes()[i % 8]))
    }

    #[inline]
    pub fn from_be_u64(be_u64: [u64; 4]) -> Self {
        Self(core::array::from_fn(|i| be_u64[i / 8].to_be_bytes()[i % 8]))
    }

    #[inline]
    pub fn to_le_u64(self) -> [u64; 4] {
        let mut ret = [[0; 8]; 4];

        for i in 0..32 {
            ret[i / 8][i % 8] = self.0[i];
        }

        core::array::from_fn(|i| u64::from_le_bytes(ret[i]))
    }

    #[inline]
    pub fn to_be_u64(self) -> [u64; 4] {
        let mut ret = [[0; 8]; 4];

        for i in 0..32 {
            ret[i / 8][i % 8] = self.0[i];
        }

        core::array::from_fn(|i| u64::from_be_bytes(ret[i]))
    }

    #[inline]
    pub fn get_identity(&self) -> String {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment =
                u64::from_le_bytes(self.0[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + b'a';
                public_key_fragment /= 26;
            }
        }

        let mut identity_bytes_checksum = [0u8; 3];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&self.0);
        kg.into_xof().squeeze(&mut identity_bytes_checksum);
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u64
            | (identity_bytes_checksum[1] as u64) << 8
            | (identity_bytes_checksum[2] as u64) << 16;
        identity_bytes_checksum &= 0x3FFFF;
        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'a';
            identity_bytes_checksum /= 26;
        }

        String::from_utf8(identity.to_vec()).unwrap()
    }
}

impl FromStr for MiningSeed {
    type Err = QubicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buffer = [0u8; 32];

        if !s
            .chars()
            .all(|c| c.is_lowercase() && c.is_ascii_alphabetic())
        {
            return Err(QubicError::InvalidIdFormatError { ident: "TxHash" });
        }

        let id = s.as_bytes();

        if id.len() != 60 {
            return Err(QubicError::InvalidIdLengthError {
                ident: "TxHash",
                expected: 60,
                found: id.len(),
            });
        }

        for i in 0..4 {
            for j in (0..14usize).rev() {
                let im = u64::from_le_bytes(buffer[i << 3..(i << 3) + 8].try_into().unwrap()) * 26
                    + (id[i * 14 + j] - b'a') as u64;
                let im = im.to_le_bytes();

                for k in 0..8 {
                    buffer[(i << 3) + k] = im[k];
                }
            }
        }

        Ok(Self(buffer))
    }
}

impl Debug for Nonce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut hex_slice = [0; 64];
        hex::encode_to_slice(self.0, &mut hex_slice).unwrap();
        f.write_str(&format!(
            "0x{}",
            String::from_utf8(hex_slice.to_vec()).unwrap()
        ))
    }
}

impl Display for Nonce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut hex_slice = [0; 64];
        hex::encode_to_slice(self.0, &mut hex_slice).unwrap();
        f.write_str(&format!(
            "0x{}",
            String::from_utf8(hex_slice.to_vec()).unwrap()
        ))
    }
}

impl Nonce {
    #[inline]
    pub fn from_le_u64(le_u64: [u64; 4]) -> Self {
        Self(core::array::from_fn(|i| le_u64[i / 8].to_le_bytes()[i % 8]))
    }

    #[inline]
    pub fn from_be_u64(be_u64: [u64; 4]) -> Self {
        Self(core::array::from_fn(|i| be_u64[i / 8].to_be_bytes()[i % 8]))
    }

    #[inline]
    pub fn to_le_u64(self) -> [u64; 4] {
        let mut ret = [[0; 8]; 4];

        for i in 0..32 {
            ret[i / 8][i % 8] = self.0[i];
        }

        core::array::from_fn(|i| u64::from_le_bytes(ret[i]))
    }

    #[inline]
    pub fn to_be_u64(self) -> [u64; 4] {
        let mut ret = [[0; 8]; 4];

        for i in 0..32 {
            ret[i / 8][i % 8] = self.0[i];
        }

        core::array::from_fn(|i| u64::from_be_bytes(ret[i]))
    }
}

impl QubicTxHash {
    #[inline]
    pub fn get_identity(&self) -> String {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment =
                u64::from_le_bytes(self.0[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + b'a';
                public_key_fragment /= 26;
            }
        }

        let mut identity_bytes_checksum = [0u8; 3];
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&self.0);
        kg.into_xof().squeeze(&mut identity_bytes_checksum);
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u64
            | (identity_bytes_checksum[1] as u64) << 8
            | (identity_bytes_checksum[2] as u64) << 16;
        identity_bytes_checksum &= 0x3FFFF;
        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'a';
            identity_bytes_checksum /= 26;
        }

        String::from_utf8(identity.to_vec()).unwrap()
    }
    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.0)
    }

    pub fn try_from_base64(s: &str) -> Result<Self, QubicError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|_| QubicError::FormattingError)?;

        if decoded.len() != 32 {
            return Err(QubicError::InvalidIdLengthError {
                ident: "TxHash",
                expected: 32,
                found: decoded.len(),
            });
        }

        let mut buffer = [0u8; 32];
        buffer.copy_from_slice(&decoded);
        Ok(Self(buffer))
    }
}

impl FromStr for QubicTxHash {
    type Err = QubicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buffer = [0u8; 32];

        if !s
            .chars()
            .all(|c| c.is_lowercase() && c.is_ascii_alphabetic())
        {
            return Err(QubicError::InvalidIdFormatError { ident: "TxHash" });
        }

        let id = s.as_bytes();

        if id.len() != 60 {
            return Err(QubicError::InvalidIdLengthError {
                ident: "TxHash",
                expected: 60,
                found: id.len(),
            });
        }

        for i in 0..4 {
            for j in (0..14usize).rev() {
                let im = u64::from_le_bytes(buffer[i << 3..(i << 3) + 8].try_into().unwrap()) * 26
                    + (id[i * 14 + j] - b'a') as u64;
                let im = im.to_le_bytes();

                for k in 0..8 {
                    buffer[(i << 3) + k] = im[k];
                }
            }
        }

        Ok(Self(buffer))
    }
}

impl Debug for QubicTxHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let id = self.get_identity();
        f.write_str(&format!("{}...{}", &id[..5], &id[55..]))
    }
}

impl Display for QubicTxHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.get_identity())
    }
}