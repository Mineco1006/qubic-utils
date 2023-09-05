#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_subborrow_u64, _addcarry_u64};

use std::ptr::copy_nonoverlapping;

use four_q::{types::PointAffine, ops::{ecc_mul_fixed, encode, decode, ecc_mul, montgomery_multiply_mod_order, ecc_mul_double}, consts::{MONTGOMERY_R_PRIME, ONE, CURVE_ORDER_0, CURVE_ORDER_1, CURVE_ORDER_3, CURVE_ORDER_2}};
use kangarootwelve::KangarooTwelve;
use thiserror::Error;

fn addcarry_u64(c_in: u8, a: u64, b: u64, out: &mut u64) -> u8  {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        _addcarry_u64(c_in, a, b, out)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        c_in
    }
}

fn subborrow_u64(c_in: u8, a: u64, b: u64, out: &mut u64) -> u8 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        _subborrow_u64(c_in, a, b, out)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        c_in
    }
}

#[derive(Debug, Error)]
pub enum QubicError {
    #[error("Invalid {ident} lenght (expected {expected}, found {found})")]
    InvalidIdLengthError { ident: &'static str, expected: usize, found: usize },

    #[error("Invalid format of {ident}. Make sure all charcters are upper/lower case")]
    InvalidIdFormatError { ident: &'static str },

    #[error("Elliptic curve error. Decoded point was not found found on the elliptic curve")]
    EllipticCurveError,

    #[error("Public key is not formatted correctly for 128bit access")]
    FormattingError
}

#[derive(Debug, Clone, Copy)]
pub struct QubicId(pub [u8; 32]);

impl QubicId {
    pub fn check_id(id: &str) -> Result<(), QubicError> {
        if !id.chars().all(|c| c.is_uppercase() && c.is_ascii_alphabetic()) {
            return Err(QubicError::InvalidIdFormatError { ident: "ID" })
        }

        let id = id.as_bytes();

        if id.len() != 60 {
            return Err(QubicError::InvalidIdLengthError { ident: "ID", expected: 55, found: id.len()})
        }

        Ok(())
    }

    pub fn from_str(id: &str) -> Result<Self, QubicError> {
        let mut buffer = [0u8; 32];

        if !id.chars().all(|c| c.is_uppercase() && c.is_ascii_alphabetic()) {
            return Err(QubicError::InvalidIdFormatError { ident: "ID" })
        }

        let id = id.as_bytes();

        if id.len() != 60 {
            return Err(QubicError::InvalidIdLengthError { ident: "ID", expected: 60, found: id.len()})
        }

        for i in 0..4 {
            for j in (0..14usize).rev() {
                let im = u64::from_le_bytes(buffer[i << 3..(i << 3) + 8].try_into().unwrap()) * 26 + (id[i * 14 + j] - b'A') as u64;
                let im = im.to_le_bytes();
                
                for k in 0..8 {
                    buffer[(i << 3) + k] = im[k];
                }
            }
        }
        

        Ok(Self(buffer))
    }

    pub fn get_identity(&self) -> String {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment = u64::from_le_bytes(self.0[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + b'A';
                public_key_fragment /= 26;
            }
        }

        let mut identity_bytes_checksum = [0u8; 3];
        let mut kg = KangarooTwelve::hash(&self.0, &[]);
        kg.squeeze(&mut identity_bytes_checksum);
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u64 | (identity_bytes_checksum[1] as u64) << 8 | (identity_bytes_checksum[1] as u64) << 16;

        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'A';
            identity_bytes_checksum /= 26;
        }

        String::from_utf8(identity.to_vec()).unwrap()
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct QubicWallet {
    private_key: [u8; 32],
    subseed: [u8; 32],
    pub public_key: [u8; 32]
}

impl QubicWallet {
    pub fn from_seed(seed: &str) -> Result<Self, QubicError> {
        let subseed = Self::get_subseed(seed)?;
        let private_key = Self::get_private_key(&subseed);
        let public_key = Self::get_public_key(&private_key);

        Ok(
            Self {
                private_key,
                public_key,
                subseed
            }
        )
    }

    pub fn get_subseed(seed: &str) -> Result<[u8; 32], QubicError> {
        if !seed.chars().all(|c| c.is_lowercase() && c.is_ascii_alphabetic()) {
            return Err(QubicError::InvalidIdFormatError { ident: "SEED" })
        }

        if seed.len() != 55 {
            return Err(QubicError::InvalidIdLengthError { ident: "SEED", expected: 55, found: seed.len()})
        }

        let seed = seed.as_bytes();
        let mut seed_bytes = [0u8; 55];

        for i in 0..55 {
            seed_bytes[i] = seed[i] - b'a';
        }

        let mut subseed = [0u8; 32];
        let mut kg = kangarootwelve::KangarooTwelve::hash(&seed_bytes, &[]);
        kg.squeeze(&mut subseed);
        

        Ok(subseed)
    }

    pub fn get_private_key(subseed: &[u8; 32]) -> [u8; 32] {
        let mut pk = [0u8; 32];
        let mut kg = KangarooTwelve::hash(subseed, &[]);
        kg.squeeze(&mut pk);

        pk
    }

    /// SchnorrQ public key generation
    /// It produces a public key publicKey, which is the encoding of P = s*G, where G is the generator and
    /// s is the output of hashing publicKey and taking the least significant 32 bytes of the result
    /// Input:  32-byte privateKey
    /// Output: 32-byte publicKey
    pub fn get_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        let mut p = PointAffine::default();
        let mut private_key = private_key.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();

        ecc_mul_fixed(&mut private_key, &mut p);
        let mut private_key: [u8; 32] = private_key.into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>().try_into().unwrap();
        encode(&mut p, &mut private_key);

        private_key
    }

    pub fn public_key_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.public_key))
    }

    pub fn get_identity(&self) -> String {
        let mut identity = [0u8; 60];
        for i in 0..4 {
            let mut public_key_fragment = u64::from_le_bytes(self.public_key[i << 3..(i << 3) + 8].try_into().unwrap());
            for j in 0..14 {
                identity[i * 14 + j] = (public_key_fragment % 26) as u8 + 'A' as u8;
                public_key_fragment /= 26;
            }
        }

        let mut identity_bytes_checksum = [0u8; 3];
        let mut kg = KangarooTwelve::hash(&self.public_key, &[]);
        kg.squeeze(&mut identity_bytes_checksum);
        let mut identity_bytes_checksum = identity_bytes_checksum[0] as u32 | (identity_bytes_checksum[1] as u32) << 8 | (identity_bytes_checksum[2] as u32) << 16;
        identity_bytes_checksum &= 0x3FFFF;

        for i in 0..4 {
            identity[56 + i] = (identity_bytes_checksum % 26) as u8 + b'A';
            identity_bytes_checksum /= 26;
        }

        String::from_utf8(identity.to_vec()).unwrap()
    }

    pub fn get_shared_key(&self) -> Result<[u8; 32], QubicError> {
        let mut a = PointAffine::default();

        if self.public_key[15] & 0x80 != 0 {
            return Err(QubicError::FormattingError)
        }

        if !decode(&self.public_key, &mut a) {
            return Err(QubicError::EllipticCurveError)
        }

        let private_key_u64 = self.private_key.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();

        unsafe {
            if !ecc_mul(&mut *(&mut a as *mut PointAffine), &private_key_u64, &mut a) {
                return Err(QubicError::EllipticCurveError)
            }
        }

        if a.x[0][0] == 0 && a.x[1][0] == 0 && a.x[0][1] == 0 && a.x[1][1] == 0 && a.y[0][0] == 0 && a.y[1][0] == 0 && a.y[0][1] == 0 && a.y[1][1] == 0 {
            return Err(QubicError::EllipticCurveError)
        }

        let mut shared_key = [0u8; 32];

        unsafe {
            copy_nonoverlapping(a.y.as_ptr() as *mut u8, shared_key.as_mut_ptr(), 32);
        }
        
        Ok(shared_key)
    }

    /// SchnorrQ signature generation
    /// It produces the signature signature of a message messageDigest of size 32 in bytes
    /// Inputs: 32-byte subseed, 32-byte publicKey, and messageDigest of size 32 in bytes
    /// Output: 64-byte signature
    pub fn sign(&self, message_digest: &[u8; 32]) -> [u8; 64] {
        let mut r_a = PointAffine::default();
        let (mut k, mut h, mut temp) = ([0u8; 64], [0u8; 64], [0u8; 96]);
        let mut r = [0u8; 64];
        let mut kg = KangarooTwelve::hash(&self.subseed, &[]);
        kg.squeeze(&mut k);

        let mut signature = [0u8; 64];

        //let mut k = k.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();
        unsafe {
            copy_nonoverlapping(k.as_ptr().offset(32), temp.as_mut_ptr().offset(32), 32);
            copy_nonoverlapping(message_digest.as_ptr(), temp.as_mut_ptr().offset(64), 32);

            let mut kg = KangarooTwelve::hash(&temp[32..], &[]);
            let mut im = [0u8; 64];
            kg.squeeze(&mut im);

            copy_nonoverlapping(im.as_ptr(), r.as_mut_ptr() as *mut u8, 64);
            let k: [u64; 8] = k.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
            let mut r: [u64; 8] = r.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
            ecc_mul_fixed(&r, &mut r_a);

            encode(&mut r_a, &mut signature);
            let mut signature_i: [u64; 8] = signature.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
            
            copy_nonoverlapping(signature_i.as_ptr() as *mut u8, temp.as_mut_ptr(), 32);
            copy_nonoverlapping(self.public_key.as_ptr(), temp.as_mut_ptr().offset(32), 32);

            let mut kg = KangarooTwelve::hash(&temp, &[]);
            kg.squeeze(&mut h);
            
            let mut h: [u64; 8] = h.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
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

            if subborrow_u64(subborrow_u64(subborrow_u64(subborrow_u64(0, r[0], signature_i[4], &mut signature_i[4]), r[1], signature_i[5], &mut signature_i[5]), r[2], signature_i[6], &mut signature_i[6]), r[3], signature_i[7], &mut signature_i[7]) != 0 {
                addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(0, signature_i[4], CURVE_ORDER_0, &mut signature_i[4]), signature_i[5], CURVE_ORDER_1, &mut signature_i[5]), signature_i[6], CURVE_ORDER_2, &mut signature_i[6]),signature_i[7], CURVE_ORDER_3, &mut signature_i[7]);
            }

            signature = signature_i.into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>().try_into().unwrap();
        }

        signature
    }

    pub fn verify(public_key: &[u8; 32], message_digest: &[u8; 32], signature: &[u8; 64]) -> bool {

        if public_key[15] & 0x80 != 0 || signature[15] & 0x80 != 0 || signature[62] & 0xC0 != 0 || signature[63] != 0 {
            return false;
        }

        let mut a = PointAffine::default();
        let (mut temp, mut h) = ([0u8; 96], [0u8; 64]);

        if !decode(public_key, &mut a) {
            return false;
        }

        //dbg!(&a);

        unsafe {
            copy_nonoverlapping(signature.as_ptr(), temp.as_mut_ptr(), 32);
            copy_nonoverlapping(public_key.as_ptr(), temp.as_mut_ptr().offset(32), 32);
            copy_nonoverlapping(message_digest.as_ptr(), temp.as_mut_ptr().offset(64), 32);
        }

        let mut signature: [u64; 8] = signature.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
        
        let mut kg = KangarooTwelve::hash(&temp, &[]);
        kg.squeeze(&mut h);


        let mut h: [u64; 8] = h.chunks_exact(8).into_iter().map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>().try_into().unwrap();
        if !ecc_mul_double(&mut signature[4..], &mut h, &mut a) {
            return false;
        }

        let mut a_bytes = [0u8; 64];

        unsafe {
            copy_nonoverlapping(&a as *const PointAffine as *mut u8, a_bytes.as_mut_ptr(), 64);
        }
        

        encode(&mut a, &mut a_bytes);

        unsafe {
            let signature_ptr = signature.as_ptr() as *const u128;
            let a_ptr = a_bytes.as_ptr() as *const u128;

            *signature_ptr == *a_ptr && *signature_ptr.offset(1) == *a_ptr.offset(1)
        }
    }
}


#[test]
pub fn test_id() {
    let id = "MMRVSTEYYUXUBELYKDEUMUEJMWEAPZSLCFYNMFAGVGUXEYNYLNFCYLPAMICH";

    let pk = QubicId::from_str(id).unwrap();

    println!("{}", pk.get_identity());

    println!("{:?}", pk.0);
}

#[test]
pub fn test_pk() {
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let subseed = QubicWallet::get_subseed(seed).unwrap();
    let private_key = QubicWallet::get_private_key(&subseed);
    let public_key1 = QubicWallet::get_public_key(&private_key);

    let wallet = QubicWallet::from_seed(seed).unwrap();
    dbg!(wallet.get_identity());
    let data = [2u8; 32];
    let signature = wallet.sign(&data);
    println!("Signature: {signature:?}");

    assert!(QubicWallet::verify(&public_key1, &data, &signature));
    dbg!(wallet.get_identity());
}