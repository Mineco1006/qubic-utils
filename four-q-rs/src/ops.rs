#![allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_subborrow_u64, _addcarry_u64};

use std::ptr::copy_nonoverlapping;
use crate::{
    types::{FelmT, F2elmT, PointPrecomp, PointExtprojPrecomp, PointAffine, PointExtproj},
    consts::{
        FIXED_BASE_TABLE,
        MONTGOMERY_SMALL_R_PRIME_0,
        MONTGOMERY_SMALL_R_PRIME_1,
        MONTGOMERY_SMALL_R_PRIME_2,
        MONTGOMERY_SMALL_R_PRIME_3,
        CURVE_ORDER,
        CURVE_ORDER_3,
        CURVE_ORDER_2,
        CURVE_ORDER_1,
        CURVE_ORDER_0,
        PARAMETER_D_F2ELM,
        MONTGOMERY_R_PRIME,
        ONE,
        C_TAU_1,
        C_TAU_DUAL_1,
        C_PHI_0,
        C_PHI_1,
        C_PHI_2,
        C_PHI_3,
        C_PHI_4,
        C_PHI_5,
        C_PHI_6,
        C_PHI_7,
        C_PHI_8,
        C_PHI_9, ELL_1, ELL_2, ELL_3, ELL_4, B11, B21, B31, B41, B12, B22, B23, B24, B32, B33, B34, B13, B14, B43, B44, B42, C1, C2, C3, C4, DOUBLE_SCALAR_TABLE, PARAMETER_D, C_PSI_2, C_PSI_1, C_PSI_3, C_PSI_4
    }};

fn addcarry_u64(c_in: u8, a: u64, b: u64, out: &mut u64) -> u8  {
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


/// Modular correction, a = a mod (2^127-1)
#[inline(always)]
pub fn mod1271(a: &mut FelmT) {
    subborrow_u64(subborrow_u64(0, a[0], 0xFFFFFFFFFFFFFFFF, &mut a[0]), a[1], 0x7FFFFFFFFFFFFFFF, &mut a[1]);
    let mask = 0 - (a[1] >> 63);
    addcarry_u64(addcarry_u64(0, a[0], mask, &mut a[0]), a[1], 0x7FFFFFFFFFFFFFFF & mask, &mut a[1]);
}

/// Field addition, c = a+b mod (2^127-1)
#[inline(always)]
pub fn fpadd1271(a: FelmT, b: FelmT, c: &mut FelmT) {
    addcarry_u64(addcarry_u64(0, a[0], b[0], &mut c[0]), a[1], b[1], &mut c[1]);
    addcarry_u64(addcarry_u64(0, c[0], c[1] >> 63, &mut c[0]), c[1] & 0x7FFFFFFFFFFFFFFF, 0, &mut c[1]);
}

/// Field subtraction, c = a-b mod (2^127-1)
#[inline(always)]
pub fn fpsub1271(a: FelmT, b: FelmT, c: &mut FelmT) {
    subborrow_u64(subborrow_u64(0, a[0], b[0], &mut c[0]), a[1], b[1], &mut c[1]);
    subborrow_u64(subborrow_u64(0, c[0], c[1] >> 63, &mut c[0]), c[1] & 0x7FFFFFFFFFFFFFFF, 0, &mut c[1]);
}

/// Field negation, a = -a mod (2^127-1)
#[inline(always)]
pub fn fpneg1271(a: &mut FelmT) {
    a[0] = !a[0];
    a[1] = 0x7FFFFFFFFFFFFFFF - a[1];
}

#[inline(always)]
pub fn _umul128(a: u64, b: u64, hi: &mut u64) -> u64 {
    let r = (a as u128) * (b as u128);
    *hi = (r >> 64) as u64;
    r as u64
}

#[inline(always)]
pub fn __shiftleft128(lo: u64, hi: u64, s: u32) -> u64 {
    let s = s % 64;
    (((lo as u128 | ((hi as u128) << 64)) << s) >> 64) as u64
}

#[inline(always)]
pub fn __shiftright128(lo: u64, hi: u64, s: u32) -> u64 {
    let s = s % 64;
    ((lo as u128 | ((hi as u128) << 64)) >> s) as u64
}

/// Field multiplication, c = a*b mod (2^127-1)
#[inline(always)]
pub fn fpmul1271(a: FelmT, b: FelmT, c: &mut FelmT) {
    let (mut tt1, mut tt2, mut tt3) = ([0u64; 2], [0u64; 2], [0u64; 2]);
    tt1[0] = _umul128(a[0], b[0], &mut tt3[0]);
    tt2[0] = _umul128(a[0], b[1], &mut tt2[1]);
    addcarry_u64(addcarry_u64(0, tt2[0], tt3[0], &mut tt2[0]), tt2[1], 0, &mut tt2[1]);
    tt3[0] = _umul128(a[1], b[0], &mut tt3[1]);
    addcarry_u64(addcarry_u64(0, tt2[0], tt3[0], &mut tt2[0]), tt2[1], tt3[1], &mut tt2[1]);
    tt3[0] = _umul128(a[1], b[1], &mut tt3[1]);
    tt3[1] = __shiftleft128(tt3[0], tt3[1], 1);
    addcarry_u64(addcarry_u64(0, __shiftright128(tt2[0], tt2[1], 63), tt3[0] << 1, &mut tt3[0]), tt2[1] >> 63, tt3[1], &mut tt3[1]);
    addcarry_u64(addcarry_u64(0, tt1[0], tt3[0], &mut tt1[0]), tt2[0] & 0x7FFFFFFFFFFFFFFF, tt3[1], &mut tt1[1]);
    addcarry_u64(addcarry_u64(0, tt1[0], tt1[1] >> 63, &mut c[0]), tt1[1] & 0x7FFFFFFFFFFFFFFF, 0, &mut c[1]);
}

/// Field squaring, c = a^2 mod (2^127-1)
#[inline(always)]
pub fn fpsqr1271(a: FelmT, c: &mut FelmT) {
    let (mut tt1, mut tt2, mut tt3) = ([0u64; 2], [0u64; 2], [0u64; 2]);
    tt1[0] = _umul128(a[0], a[0], &mut tt3[0]);
    tt2[0] = _umul128(a[0], a[1], &mut tt2[1]);
    addcarry_u64(addcarry_u64(0, tt2[0], tt3[0], &mut tt3[0]), tt2[1], 0, &mut tt3[1]);
    addcarry_u64(addcarry_u64(0, tt2[0], tt3[0], &mut tt2[0]), tt2[1], tt3[1], &mut tt2[1]);
    tt3[0] = _umul128(a[1], a[1], &mut tt3[1]);
    tt3[1] = __shiftleft128(tt3[0], tt3[1], 1);
    addcarry_u64(addcarry_u64(0, __shiftright128(tt2[0], tt2[1], 63), tt3[0] << 1, &mut tt3[0]), tt2[1] >> 63, tt3[1], &mut tt3[1]);
    addcarry_u64(addcarry_u64(0, tt1[0], tt3[0], &mut tt1[0]), tt2[0] & 0x7FFFFFFFFFFFFFFF, tt3[1], &mut tt1[1]);
    addcarry_u64(addcarry_u64(0, tt1[0], tt1[1] >> 63, &mut c[0]), tt1[1] & 0x7FFFFFFFFFFFFFFF, 0, &mut c[1]);
}

/// Field squaring, c = a^2 mod (2^127-1)
#[inline(always)]
pub fn fpexp1251(a: FelmT, af: &mut FelmT) {
    let (mut t1, mut t2, mut t3, mut t4, mut t5) = ([0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2]);

    fpsqr1271(a, &mut t2);
    fpmul1271(a, t2, &mut t2);
    fpsqr1271(t2, &mut t3);
    fpsqr1271(t3, &mut t3);
    fpmul1271(t2, t3, &mut t3);
    fpsqr1271(t3, &mut t4);
    fpsqr1271(t4, &mut t4);
    fpsqr1271(t4, &mut t4);
    fpsqr1271(t4, &mut t4);
    fpmul1271(t3, t4, &mut t4);
    fpsqr1271(t4, &mut t5);
    for _ in 0..7  {
        fpsqr1271(t5, &mut t5)
    }
    fpmul1271(t4, t5, &mut t5);
    fpsqr1271(t5, &mut t2);
    for _ in 0..15 {
        fpsqr1271(t2, &mut t2);
    }
    fpmul1271(t5, t2, &mut t2);
    fpsqr1271(t2, &mut t1);
    for _ in 0..31 {
        fpsqr1271(t1, &mut t1)
    }
    fpmul1271(t2, t1, &mut t1);
    for _ in 0..32 {
        fpsqr1271(t1, &mut t1)
    }
    //for (unsigned int i = 0; i < 32; i++) fpsqr1271(t1, t1);
    fpmul1271(t1, t2, &mut t1);

    for _ in 0..16 {
        fpsqr1271(t1, &mut t1)
    }
    //for (unsigned int i = 0; i < 16; i++) fpsqr1271(t1, t1);
    fpmul1271(t5, t1, &mut t1);

    for _ in 0..8 {
        fpsqr1271(t1, &mut t1)
    }
    //for (unsigned int i = 0; i < 8; i++) fpsqr1271(t1, t1);
    fpmul1271(t4, t1, &mut t1);
    for _ in 0..4 {
        fpsqr1271(t1, &mut t1)
    }
    //for (unsigned int i = 0; i < 4; i++) fpsqr1271(t1, t1);
    fpmul1271(t3, t1, &mut t1);
    fpsqr1271(t1, &mut t1);
    fpmul1271(a, t1, af);
}

/// GF(p^2) division by two c = a/2 mod p
#[inline(always)]
pub fn fp2div1271(a: &mut F2elmT) {
    let mut mask: u64;
    let mut temp = [0u64; 2];

    mask = 0 - (1 & a[0][0]);
    addcarry_u64(addcarry_u64(0, a[0][0], mask, &mut temp[0]), a[0][1], mask >> 1, &mut temp[1]);
    a[0][0] = __shiftright128(temp[0], temp[1], 1);
    a[0][1] = temp[1] >> 1;

    mask = 0 - (1 & a[1][0]);
    addcarry_u64(addcarry_u64(0, a[1][0], mask, &mut temp[0]), a[1][1], mask >> 1, &mut temp[1]);
    a[1][0] = __shiftright128(temp[0], temp[1], 1);
    a[1][1] = temp[1] >> 1;
}

/// GF(p^2) negation, a = -a in GF((2^127-1)^2)
#[inline(always)]
pub fn fp2neg1271(a: &mut F2elmT) {
    fpneg1271(&mut a[0]);
    fpneg1271(&mut a[1]);
}

/// GF(p^2) squaring, c = a^2 in GF((2^127-1)^2)
#[inline(always)]
pub fn fp2sqr1271(a: F2elmT, c: &mut F2elmT) {
    let (mut t1, mut t2, mut t3) = ([0u64; 2], [0u64; 2], [0u64; 2]);
    
    fpadd1271(a[0], a[1], &mut t1);           // t1 = a0+a1 
    fpsub1271(a[0], a[1], &mut t2);           // t2 = a0-a1
    fpmul1271(a[0], a[1], &mut t3);           // t3 = a0*a1
    fpmul1271(t1, t2, &mut c[0]);             // c0 = (a0+a1)(a0-a1)
    fpadd1271(t3, t3, &mut c[1]);             // c1 = 2a0*a1
}

/// GF(p^2) multiplication, c = a*b in GF((2^127-1)^2)
#[inline(always)]
pub fn fp2mul1271(a: F2elmT, b: F2elmT, c: &mut F2elmT) {
    let (mut t1, mut t2, mut t3, mut t4) = ([0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2]);

    fpmul1271(a[0], b[0], &mut t1);          // t1 = a0*b0
    fpmul1271(a[1], b[1], &mut t2);          // t2 = a1*b1
    fpadd1271(a[0], a[1], &mut t3);          // t3 = a0+a1
    fpadd1271(b[0], b[1], &mut t4);          // t4 = b0+b1
    fpsub1271(t1, t2, &mut c[0]);            // c[0] = a0*b0 - a1*b1
    fpmul1271(t3, t4, &mut t3);              // t3 = (a0+a1)*(b0+b1)
    fpsub1271(t3, t1, &mut t3);              // t3 = (a0+a1)*(b0+b1) - a0*b0
    fpsub1271(t3, t2, &mut c[1]);            // c[1] = (a0+a1)*(b0+b1) - a0*b0 - a1*b1  
}

/// GF(p^2) addition, c = a+b in GF((2^127-1)^2)
#[inline(always)]
pub fn fp2add1271(a: F2elmT, b: F2elmT, c: &mut F2elmT) {
    fpadd1271(a[0], b[0], &mut c[0]);
    fpadd1271(a[1], b[1], &mut c[1]);
}

/// GF(p^2) subtraction, c = a-b in GF((2^127-1)^2) 
#[inline(always)]
pub fn fp2sub1271(a: F2elmT, b: F2elmT, c: &mut F2elmT) {
    fpsub1271(a[0], b[0], &mut c[0]);
    fpsub1271(a[1], b[1], &mut c[1]);
}

/// GF(p^2) addition followed by subtraction, c = 2a-b in GF((2^127-1)^2)
#[inline(always)]
pub fn fp2addsub1271(mut a: F2elmT, b: F2elmT, c: &mut F2elmT) {
    fp2add1271(a, a, &mut a);
    fp2sub1271(a, b, c);
}

/// Table lookup to extract a point represented as (x+y,y-x,2t) corresponding to extended twisted Edwards coordinates (X:Y:Z:T) with Z=1
#[inline]
pub unsafe fn table_lookup_fixed_base(p: &mut PointPrecomp, digit: u64, sign: u64) {
    let digit = digit as isize;
    if sign != 0 {
        //let table_pointer = *(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit);
        p.xy.copy_from_slice(&(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).yx);
        p.yx.copy_from_slice(&(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).xy);
        p.t2[0][0] = !(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).t2[0][0];
        p.t2[0][1] = 0x7FFFFFFFFFFFFFFF - (*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).t2[0][1];
        p.t2[1][0] = !(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).t2[1][0];
        p.t2[1][1] = 0x7FFFFFFFFFFFFFFF - (*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).t2[1][1];
    } else {
        p.xy.copy_from_slice(&(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).xy);
        p.yx.copy_from_slice(&(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).yx);
        p.t2.copy_from_slice(&(*(FIXED_BASE_TABLE.as_ptr() as *const PointPrecomp).offset(digit)).t2);
    }
}

#[inline]
pub fn multiply(a: &[u64], b: &[u64], c: &mut [u64]) {
    let (mut u, mut v, mut uv) = (0, 0, 0);
    c[0] = _umul128(a[0], b[0], &mut u);
    u = addcarry_u64(0, _umul128(a[0], b[1], &mut uv), u, &mut c[1]) as u64 + uv;
    u = addcarry_u64(0, _umul128(a[0], b[2], &mut uv), u, &mut c[2]) as u64 + uv;
    c[4] = addcarry_u64(0, _umul128(a[0], b[3], &mut uv), u, &mut c[3]) as u64 + uv;

    u = addcarry_u64(0, c[1], _umul128(a[1], b[0], &mut uv), &mut c[1]) as u64 + uv;
    u = addcarry_u64(0, _umul128(a[1], b[1], &mut uv), u, &mut v) as u64 + uv;
    u = addcarry_u64(addcarry_u64(0, c[2], v, &mut c[2]), _umul128(a[1], b[2], &mut uv), u, &mut v) as u64 + uv;
    c[5] = addcarry_u64(addcarry_u64(0, c[3], v, &mut c[3]), _umul128(a[1], b[3], &mut uv), u, &mut v) as u64 + uv + addcarry_u64(0, c[4], v, &mut c[4]) as u64;

    u = addcarry_u64(0, c[2], _umul128(a[2], b[0], &mut uv), &mut c[2]) as u64 + uv;
    u = addcarry_u64(0, _umul128(a[2], b[1], &mut uv), u, &mut v) as u64 + uv;
    u = addcarry_u64(addcarry_u64(0, c[3], v, &mut c[3]), _umul128(a[2], b[2], &mut uv), u, &mut v) as u64 + uv;
    c[6] = addcarry_u64(addcarry_u64(0, c[4], v, &mut c[4]), _umul128(a[2], b[3], &mut uv), u, &mut v) as u64 + uv + addcarry_u64(0, c[5], v, &mut c[5]) as u64;

    u = addcarry_u64(0, c[3], _umul128(a[3], b[0], &mut uv), &mut c[3]) as u64 + uv;
    u = addcarry_u64(0, _umul128(a[3], b[1], &mut uv), u, &mut v) as u64 + uv;
    u = addcarry_u64(addcarry_u64(0, c[4], v, &mut c[4]), _umul128(a[3], b[2], &mut uv), u, &mut v) as u64 + uv;
    c[7] = addcarry_u64(addcarry_u64(0, c[5], v, &mut c[5]), _umul128(a[3], b[3], &mut uv), u, &mut v) as u64 + uv + addcarry_u64(0, c[6], v, &mut c[6]) as u64;
}

/// 256-bit Montgomery multiplication modulo the curve order, mc = ma*mb*r' mod order, where ma,mb,mc in [0, order-1]
/// ma, mb and mc are assumed to be in Montgomery representation
/// The Montgomery constant r' = -r^(-1) mod 2^(log_2(r)) is the global value "Montgomery_rprime", where r is the order 
#[inline]
pub fn montgomery_multiply_mod_order(ma: &[u64], mb: &[u64], mc: &mut [u64]) {
    let mut p = [0u64; 8];
    let mut q = [0u64; 4];
    let mut temp = [0u64; 8];

    unsafe {
        if mb[0] == 1 && !mb[1] != 0 && !mb[2] != 0 && !mb[3] != 0 {
            copy_nonoverlapping(ma.as_ptr(), p.as_mut_ptr(), 4);
        } else {
            multiply(ma, mb, &mut p);
        }

        let (mut u, mut v, mut uv) = (0u64, 0u64, 0u64);

        q[0] = _umul128(p[0], MONTGOMERY_SMALL_R_PRIME_0, &mut u);
        u = addcarry_u64(0, _umul128(p[0], MONTGOMERY_SMALL_R_PRIME_1, &mut uv), u, &mut q[1]) as u64 + uv;
        u = addcarry_u64(0, _umul128(p[0], MONTGOMERY_SMALL_R_PRIME_2, &mut uv), u, &mut q[2]) as u64 + uv;
        addcarry_u64(0, p[0] * MONTGOMERY_SMALL_R_PRIME_3, u, &mut q[3]);
        u = addcarry_u64(0, q[1], _umul128(p[1], MONTGOMERY_SMALL_R_PRIME_0, &mut uv), &mut q[1]) as u64 + uv;
        u = addcarry_u64(0, _umul128(p[1], MONTGOMERY_SMALL_R_PRIME_1, &mut uv), u, &mut v) as u64 + uv;
        addcarry_u64(addcarry_u64(0, q[2], v, &mut q[2]), p[1] * MONTGOMERY_SMALL_R_PRIME_2, u, &mut v);
        addcarry_u64(0, q[3], v, &mut q[3]);
        u = addcarry_u64(0, q[2], _umul128(p[2], MONTGOMERY_SMALL_R_PRIME_0, &mut uv), &mut q[2]) as u64 + uv;
        addcarry_u64(0, p[2] * MONTGOMERY_SMALL_R_PRIME_1, u, &mut v);
        addcarry_u64(0, q[3], v, &mut q[3]);
        addcarry_u64(0, q[3], p[3] * MONTGOMERY_SMALL_R_PRIME_0, &mut q[3]);

        multiply(&q, &CURVE_ORDER, &mut temp); // temp = Q * r

        let a = addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(0, p[0], temp[0], &mut temp[0]), p[1], temp[1], &mut temp[1]), p[2], temp[2], &mut temp[2]), p[3], temp[3], &mut temp[3]), p[4], temp[4], &mut temp[4]), p[5], temp[5], &mut temp[5]), p[6], temp[6], &mut temp[6]), p[7], temp[7], &mut temp[7]);
        let b = subborrow_u64(subborrow_u64(subborrow_u64(subborrow_u64(0, temp[4], CURVE_ORDER_0, &mut mc[0]), temp[5], CURVE_ORDER_1, &mut mc[1]), temp[6], CURVE_ORDER_2, &mut mc[2]), temp[7], CURVE_ORDER_3, &mut mc[3]);

        // temp not correct after addcarry
        if a - b != 0
        {
            addcarry_u64(addcarry_u64(addcarry_u64(addcarry_u64(0, mc[0], CURVE_ORDER_0, &mut mc[0]), mc[1], CURVE_ORDER_1, &mut mc[1]), mc[2], CURVE_ORDER_2, &mut mc[2]), mc[3], CURVE_ORDER_3, &mut mc[3]);
        }
    }
}

/// Normalize a projective point (X1:Y1:Z1), including full reduction
#[inline]
pub fn eccnorm(p: &mut PointExtproj, q: &mut PointAffine) {
    let mut t1 = [[0u64; 2]; 2];

    fpsqr1271(p.z[0], &mut t1[0]);
    fpsqr1271(p.z[1], &mut t1[1]);
    fpadd1271(t1[0], t1[1], &mut t1[0]);
    fpexp1251(t1[0], &mut t1[1]);
    fpsqr1271(t1[1], &mut t1[1]);
    fpsqr1271(t1[1], &mut t1[1]);
    fpmul1271(t1[0], t1[1], &mut t1[0]);
    fpneg1271(&mut p.z[1]);
    fpmul1271(p.z[0], t1[0], &mut p.z[0]);
    fpmul1271(p.z[1], t1[0], &mut p.z[1]);

    fp2mul1271(p.x, p.z, &mut q.x);          // X1 = X1/Z1
    fp2mul1271(p.y, p.z, &mut q.y);          // Y1 = Y1/Z1
    mod1271(&mut q.x[0]);
    mod1271(&mut q.x[1]);
    mod1271(&mut q.y[0]);
    mod1271(&mut q.y[1]);
}

/// Conversion from representation (X,Y,Z,Ta,Tb) to (X+Y,Y-X,2Z,2dT), where T = Ta*Tb
#[inline]
pub fn r1_to_r2(p: &PointExtproj, q: &mut PointExtprojPrecomp) {
    fp2add1271(p.ta, p.ta, &mut q.t2);                  // T = 2*Ta
    fp2add1271(p.x, p.y, &mut q.xy);                    // QX = X+Y
    fp2sub1271(p.y, p.x, &mut q.yx);                    // QY = Y-X 
    fp2mul1271(q.t2, p.tb, &mut q.t2);                  // T = 2*T
    fp2add1271(p.z, p.z, &mut q.z2);                    // QZ = 2*Z
    fp2mul1271(q.t2, PARAMETER_D_F2ELM, &mut q.t2);  // QT = 2d*T
}

/// Conversion from representation (X,Y,Z,Ta,Tb) to (X+Y,Y-X,Z,T), where T = Ta*Tb
#[inline]
pub fn r1_to_r3(p: &PointExtproj, q: &mut PointExtprojPrecomp) {
    fp2add1271(p.x, p.y, &mut q.xy);         // XQ = (X1+Y1) 
    fp2sub1271(p.y, p.x, &mut q.yx);         // YQ = (Y1-X1) 
    fp2mul1271(p.ta, p.tb, &mut q.t2);       // TQ = T1

    unsafe {
        copy_nonoverlapping(p.z.as_ptr() as *mut u64, q.z2.as_mut_ptr() as *mut u64, 4) // ZQ = Z1 
    }         
}

/// Conversion from representation (X+Y,Y-X,2Z,2dT) to (2X,2Y,2Z,2dT) 
#[inline]
pub fn r2_to_r4(p: &PointExtprojPrecomp, q: &mut PointExtproj) {
    fp2sub1271(p.xy, p.yx, &mut q.x);        // XQ = 2*X1
    fp2add1271(p.xy, p.yx, &mut q.y);        // YQ = 2*Y1

    unsafe {
        copy_nonoverlapping(p.z2.as_ptr() as *mut u64, q.z.as_mut_ptr() as *mut u64, 4) // ZQ = Z1 
    }  
}

// Point doubling 2P
#[inline]
pub fn eccdouble(p: &mut PointExtproj) {
    let (mut t1, mut t2) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2sqr1271(p.x, &mut t1);                  // t1 = X1^2
    fp2sqr1271(p.y, &mut t2);                  // t2 = Y1^2
    fp2add1271(p.x, p.y, &mut p.x);          // t3 = X1+Y1
    fp2add1271(t1, t2, &mut p.tb);             // Tbfinal = X1^2+Y1^2      
    fp2sub1271(t2, t1, &mut t1);                // t1 = Y1^2-X1^2      
    fp2sqr1271(p.x, &mut p.ta);               // Ta = (X1+Y1)^2 
    fp2sqr1271(p.z, &mut t2);                  // t2 = Z1^2  
    fp2sub1271(p.ta, p.tb, &mut p.ta);       // Tafinal = 2X1*Y1 = (X1+Y1)^2-(X1^2+Y1^2)  

    /*fp2add1271(t2, t2, &mut t2);
    fp2sub1271(t2, t1, &mut t2);*/
    fp2addsub1271(t2, t1, &mut t2);

    fp2mul1271(t1, p.tb, &mut p.y);           // Yfinal = (X1^2+Y1^2)(Y1^2-X1^2)  
    fp2mul1271(t2, p.ta, &mut p.x);           // Xfinal = 2X1*Y1*[2Z1^2-(Y1^2-X1^2)]
    fp2mul1271(t1, t2, &mut p.z);              // Zfinal = (Y1^2-X1^2)[2Z1^2-(Y1^2-X1^2)]
}

/// Basic point addition r = P+Q or r = P+P
#[inline]
pub fn eccadd_core(p: &PointExtprojPrecomp, q: &PointExtprojPrecomp, r: &mut PointExtproj) {
    let (mut t1, mut t2) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2mul1271(p.t2, q.t2, &mut r.z);        // Z = 2dT1*T2 
    fp2mul1271(p.z2, q.z2, &mut t1);          // t1 = 2Z1*Z2  
    fp2mul1271(p.xy, q.xy, &mut r.x);        // X = (X1+Y1)(X2+Y2) 
    fp2mul1271(p.yx, q.yx, &mut r.y);        // Y = (Y1-X1)(Y2-X2) 
    fp2sub1271(t1, r.z, &mut t2);              // t2 = theta
    fp2add1271(t1, r.z, &mut t1);              // t1 = alpha
    fp2sub1271(r.x, r.y, &mut r.tb);         // Tbfinal = beta
    fp2add1271(r.x, r.y, &mut r.ta);         // Tafinal = omega
    fp2mul1271(r.tb, t2, &mut r.x);           // Xfinal = beta*theta
    fp2mul1271(t1, t2, &mut r.z);              // Zfinal = theta*alpha
    fp2mul1271(r.ta, t1, &mut r.y);           // Yfinal = alpha*omega
}

/// Complete point addition P = P+Q or P = P+P
#[inline]
pub fn eccadd(q: &PointExtprojPrecomp, p: &mut PointExtproj) {
    let mut r = PointExtprojPrecomp::default();

    r1_to_r3(p, &mut r);
    eccadd_core(q, &r, p);
}

/// Point conversion to representation (X,Y,Z,Ta,Tb)
#[inline]
pub fn point_setup(p: &PointAffine, q: &mut PointExtproj) {
    unsafe {
        copy_nonoverlapping(p.x.as_ptr(), q.x.as_mut_ptr(), 2);
        copy_nonoverlapping(p.y.as_ptr(), q.y.as_mut_ptr(), 2);
        copy_nonoverlapping(p.x.as_ptr(), q.ta.as_mut_ptr(), 2);
        copy_nonoverlapping(p.y.as_ptr(), q.tb.as_mut_ptr(), 2);
        
        q.z[0][0] = 1;
        q.z[0][1] = 0;
        q.z[1][0] = 0;
        q.z[1][1] = 0;
    }
}

/// Point validation: check if point lies on the curve
#[inline]
pub fn ecc_point_validate(p: &PointExtproj) -> bool {
    let (mut t1, mut t2, mut t3) = ([[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2sqr1271(p.y, &mut t1);
    fp2sqr1271(p.x, &mut t2);
    fp2sub1271(t1, t2, &mut t3);                                 // -x^2 + y^2 
    fp2mul1271(t1, t2, &mut t1);                                 // x^2*y^2
    fp2mul1271(t1, PARAMETER_D_F2ELM, &mut t2);              // dx^2*y^2
    t1[0][0] = 1;
    t1[0][1] = 0;
    t1[1][0] = 0;
    t1[1][1] = 0; // t1 = 1
    fp2add1271(t2, t1, &mut t2);                                 // 1 + dx^2*y^2
    fp2sub1271(t3, t2, &mut t1);                                 // -x^2 + y^2 - 1 - dx^2*y^2

    (!((t1[0][0] | t1[0][1]) != 0) || !(((t1[0][0] + 1) | (t1[0][1] + 1)) != 0)) && (!((t1[1][0] | t1[1][1]) != 0)|| !(((t1[1][0] + 1) | (t1[1][1] + 1)) != 0))
}

/// Mixed point addition P = P+Q or P = P+P
#[inline]
pub fn eccmadd(q: &PointPrecomp, p: &mut PointExtproj) {
    let (mut t1, mut t2) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2mul1271(p.ta, p.tb, &mut p.ta);        // Ta = T1
    fp2add1271(p.z, p.z, &mut t1);             // t1 = 2Z1        
    fp2mul1271(p.ta, q.t2, &mut p.ta);        // Ta = 2dT1*t2 
    fp2add1271(p.x, p.y, &mut p.z);           // Z = (X1+Y1) 
    fp2sub1271(p.y, p.x, &mut p.tb);          // Tb = (Y1-X1)
    fp2sub1271(t1, p.ta, &mut t2);              // t2 = theta
    fp2add1271(t1, p.ta, &mut t1);              // t1 = alpha
    fp2mul1271(q.xy, p.z, &mut p.ta);         // Ta = (X1+Y1)(x2+y2)
    fp2mul1271(q.yx, p.tb, &mut p.x);         // X = (Y1-X1)(y2-x2)
    fp2mul1271(t1, t2, &mut p.z);               // Zfinal = theta*alpha
    fp2sub1271(p.ta, p.x, &mut p.tb);         // Tbfinal = beta
    fp2add1271(p.ta, p.x, &mut p.ta);         // Tafinal = omega
    fp2mul1271(p.tb, t2, &mut p.x);            // Xfinal = beta*theta
    fp2mul1271(p.ta, t1, &mut p.y);            // Yfinal = alpha*omega
}

/// Fixed-base scalar multiplication Q = k*G, where G is the generator. FIXED_BASE_TABLE stores v*2^(w-1) = 80 multiples of G.
#[inline]
pub fn ecc_mul_fixed(k: &[u64], q: &mut PointAffine) {
    let mut digits = [0u64; 250];
    let mut scalar = [0u64; 4];

    montgomery_multiply_mod_order(k, &MONTGOMERY_R_PRIME, &mut scalar);
    let scalar1 = scalar;
    montgomery_multiply_mod_order(&scalar1, &ONE, &mut scalar);

    unsafe {
        if scalar[0] & 1 == 0 {
            let mut carry = addcarry_u64(0, scalar[0], CURVE_ORDER_0, &mut scalar[0]);
            carry = addcarry_u64(carry, scalar[1], CURVE_ORDER_1, &mut scalar[1]);
            carry = addcarry_u64(carry, scalar[2], CURVE_ORDER_2, &mut scalar[2]);
            addcarry_u64(carry, scalar[3], CURVE_ORDER_3, &mut scalar[3]);
        }

        scalar[0] = __shiftright128(scalar[0], scalar[1], 1);
        scalar[1] = __shiftright128(scalar[1], scalar[2], 1);
        scalar[2] = __shiftright128(scalar[2], scalar[3], 1);
        scalar[3] >>= 1;

        for i in 0..49 {
            digits[i] = (scalar[0] & 1) - 1;  // Convention for the "sign" row: if scalar_(i+1) = 0 then digit_i = -1 (negative), else if scalar_(i+1) = 1 then digit_i = 0 (positive)

            // Shift scalar to the right by 1   
            scalar[0] = __shiftright128(scalar[0], scalar[1], 1);
            scalar[1] = __shiftright128(scalar[1], scalar[2], 1);
            scalar[2] = __shiftright128(scalar[2], scalar[3], 1);
            scalar[3] >>= 1;
        }

        for i in 50..250 {
            digits[i] = scalar[0] & 1;

            // Shift scalar to the right by 1
            scalar[0] = __shiftright128(scalar[0], scalar[1], 1);
            scalar[1] = __shiftright128(scalar[1], scalar[2], 1);
            scalar[2] = __shiftright128(scalar[2], scalar[3], 1);
            scalar[3] >>= 1;

            let temp = (0 - digits[i - (i / 50) * 50]) & digits[i];

            scalar[0] += temp;
            let mut carry = if scalar[0] != 0 { 0 } else { temp & 1};
            scalar[1] += carry;
            carry = if scalar[1] != 0 { 0 } else { carry & 1 };
            scalar[2] += carry;
            scalar[3] += if scalar[2] != 0 { 0 } else { carry & 1 };
        }

        let mut r = PointExtproj::default();
        let mut s = PointPrecomp::default();

        table_lookup_fixed_base(&mut s, 64 + (((((digits[249] << 1) + digits[199]) << 1) + digits[149]) << 1) + digits[99], 0);
        // Conversion from representation (x+y,y-x,2dt) to (X,Y,Z,Ta,Tb) 
        fp2sub1271(s.xy, s.yx, &mut r.x);                                 // 2*x1
        fp2add1271(s.xy, s.yx, &mut r.y);                                 // 2*y1
        fp2div1271(&mut r.x);                                               // XQ = x1
        fp2div1271(&mut r.y);                                               // YQ = y1 
        r.z[0][0] = 1;
        r.z[0][1] = 0;
        r.z[1][0] = 0;
        r.z[1][1] = 0; // ZQ = 1
        copy_nonoverlapping(r.x.as_ptr(), r.ta.as_mut_ptr(), 2);
        copy_nonoverlapping(r.y.as_ptr(), r.tb.as_mut_ptr(), 2);


        table_lookup_fixed_base(&mut s, 48 + (((((digits[239] << 1) + digits[189]) << 1) + digits[139]) << 1) + digits[89], digits[39]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[229] << 1) + digits[179]) << 1) + digits[129]) << 1) + digits[79], digits[29]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[219] << 1) + digits[169]) << 1) + digits[119]) << 1) + digits[69], digits[19]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[209] << 1) + digits[159]) << 1) + digits[109]) << 1) + digits[59], digits[9]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[248] << 1) + digits[198]) << 1) + digits[148]) << 1) + digits[98], digits[48]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[238] << 1) + digits[188]) << 1) + digits[138]) << 1) + digits[88], digits[38]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[228] << 1) + digits[178]) << 1) + digits[128]) << 1) + digits[78], digits[28]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[218] << 1) + digits[168]) << 1) + digits[118]) << 1) + digits[68], digits[18]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[208] << 1) + digits[158]) << 1) + digits[108]) << 1) + digits[58], digits[8]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[247] << 1) + digits[197]) << 1) + digits[147]) << 1) + digits[97], digits[47]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[237] << 1) + digits[187]) << 1) + digits[137]) << 1) + digits[87], digits[37]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[227] << 1) + digits[177]) << 1) + digits[127]) << 1) + digits[77], digits[27]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[217] << 1) + digits[167]) << 1) + digits[117]) << 1) + digits[67], digits[17]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[207] << 1) + digits[157]) << 1) + digits[107]) << 1) + digits[57], digits[7]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[246] << 1) + digits[196]) << 1) + digits[146]) << 1) + digits[96], digits[46]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[236] << 1) + digits[186]) << 1) + digits[136]) << 1) + digits[86], digits[36]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[226] << 1) + digits[176]) << 1) + digits[126]) << 1) + digits[76], digits[26]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[216] << 1) + digits[166]) << 1) + digits[116]) << 1) + digits[66], digits[16]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[206] << 1) + digits[156]) << 1) + digits[106]) << 1) + digits[56], digits[6]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[245] << 1) + digits[195]) << 1) + digits[145]) << 1) + digits[95], digits[45]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[235] << 1) + digits[185]) << 1) + digits[135]) << 1) + digits[85], digits[35]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[225] << 1) + digits[175]) << 1) + digits[125]) << 1) + digits[75], digits[25]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[215] << 1) + digits[165]) << 1) + digits[115]) << 1) + digits[65], digits[15]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[205] << 1) + digits[155]) << 1) + digits[105]) << 1) + digits[55], digits[5]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[244] << 1) + digits[194]) << 1) + digits[144]) << 1) + digits[94], digits[44]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[234] << 1) + digits[184]) << 1) + digits[134]) << 1) + digits[84], digits[34]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[224] << 1) + digits[174]) << 1) + digits[124]) << 1) + digits[74], digits[24]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[214] << 1) + digits[164]) << 1) + digits[114]) << 1) + digits[64], digits[14]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[204] << 1) + digits[154]) << 1) + digits[104]) << 1) + digits[54], digits[4]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[243] << 1) + digits[193]) << 1) + digits[143]) << 1) + digits[93], digits[43]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[233] << 1) + digits[183]) << 1) + digits[133]) << 1) + digits[83], digits[33]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[223] << 1) + digits[173]) << 1) + digits[123]) << 1) + digits[73], digits[23]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[213] << 1) + digits[163]) << 1) + digits[113]) << 1) + digits[63], digits[13]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[203] << 1) + digits[153]) << 1) + digits[103]) << 1) + digits[53], digits[3]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[242] << 1) + digits[192]) << 1) + digits[142]) << 1) + digits[92], digits[42]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[232] << 1) + digits[182]) << 1) + digits[132]) << 1) + digits[82], digits[32]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[222] << 1) + digits[172]) << 1) + digits[122]) << 1) + digits[72], digits[22]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[212] << 1) + digits[162]) << 1) + digits[112]) << 1) + digits[62], digits[12]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[202] << 1) + digits[152]) << 1) + digits[102]) << 1) + digits[52], digits[2]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[241] << 1) + digits[191]) << 1) + digits[141]) << 1) + digits[91], digits[41]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[231] << 1) + digits[181]) << 1) + digits[131]) << 1) + digits[81], digits[31]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[221] << 1) + digits[171]) << 1) + digits[121]) << 1) + digits[71], digits[21]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[211] << 1) + digits[161]) << 1) + digits[111]) << 1) + digits[61], digits[11]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[201] << 1) + digits[151]) << 1) + digits[101]) << 1) + digits[51], digits[1]);
        eccmadd(&s, &mut r);

        eccdouble(&mut r);
        table_lookup_fixed_base(&mut s, 64 + (((((digits[240] << 1) + digits[190]) << 1) + digits[140]) << 1) + digits[90], digits[40]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 48 + (((((digits[230] << 1) + digits[180]) << 1) + digits[130]) << 1) + digits[80], digits[30]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 32 + (((((digits[220] << 1) + digits[170]) << 1) + digits[120]) << 1) + digits[70], digits[20]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 16 + (((((digits[210] << 1) + digits[160]) << 1) + digits[110]) << 1) + digits[60], digits[10]);
        eccmadd(&s, &mut r);
        table_lookup_fixed_base(&mut s, 00 + (((((digits[200] << 1) + digits[150]) << 1) + digits[100]) << 1) + digits[50], digits[0]);
        eccmadd(&s, &mut r);

        eccnorm(&mut r, q);
    }
}

pub const fn f2elm_from_array(a: [u64; 4]) -> [[u64; 2]; 2] {
    [[a[0], a[1]], [a[2], a[3]]]
}

/// Apply tau_dual mapping to a point, P = tau_dual(P)
#[inline]
pub fn ecc_tau(p: &mut PointExtproj) {
    let (mut t0, mut t1) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2sqr1271(p.x, &mut t0);                     // t0 = X1^2
    fp2sqr1271(p.y, &mut t1);                     // t1 = Y1^2
    fp2mul1271(p.x, p.y, &mut p.x);             // X = X1*Y1
    fp2sqr1271(p.z, &mut p.y);                   // Y = Z1^2
    fp2add1271(t0, t1, &mut p.z);                 // Z = X1^2+Y1^2
    fp2sub1271(t1, t0, &mut t0);                   // t0 = Y1^2-X1^2
    fp2add1271(p.y, p.y, &mut p.y);             // Y = 2*Z1^2
    fp2mul1271(p.x, t0, &mut p.x);               // X = X1*Y1*(Y1^2-X1^2)
    fp2sub1271(p.y, t0, &mut p.y);               // Y = 2*Z1^2-(Y1^2-X1^2)
    fp2mul1271(p.x, f2elm_from_array(C_TAU_1), &mut p.x);  // Xfinal = X*ctau1
    fp2mul1271(p.y, p.z, &mut p.y);             // Yfinal = Y*Z
    fp2mul1271(p.z, t0, &mut p.z);               // Zfinal = t0*Z
}

/// Apply tau_dual mapping to a point, P = tau_dual(P)
#[inline]
pub fn ecc_tau_dual(p: &mut PointExtproj) {
    let (mut t0, mut t1) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2sqr1271(p.x, &mut t0);                          // t0 = X1^2
    fp2sqr1271(p.z, &mut p.ta);                       // Ta = Z1^2
    fp2sqr1271(p.y, &mut t1);                          // t1 = Y1^2
    fp2add1271(p.ta, p.ta, &mut p.z);                // Z = 2*Z1^2
    fp2sub1271(t1, t0, &mut p.ta);                     // Tafinal = Y1^2-X1^2
    fp2add1271(t0, t1, &mut t0);                        // t0 = X1^2+Y1^2
    fp2mul1271(p.x, p.y, &mut p.x);                  // X = X1*Y1
    fp2sub1271(p.z, p.ta, &mut p.z);                 // Z = 2*Z1^2-(Y1^2-X1^2)
    fp2mul1271(p.x, f2elm_from_array(C_TAU_DUAL_1), &mut p.tb);  // Tbfinal = ctaudual1*X1*X1
    fp2mul1271(p.z, p.ta, &mut p.y);                 // Yfinal = Z*Tafinal
    fp2mul1271(p.tb, t0, &mut p.x);                   // Xfinal = Tbfinal*t0
    fp2mul1271(p.z, t0, &mut p.z);                    // Zfinal = Z*t0
}

/// Apply delta_phi_delta mapping to a point, P = delta(phi_W(delta_inv(P))), 
/// where phi_W is the endomorphism on the Weierstrass form
#[inline]
pub fn ecc_delphidel(p: &mut PointExtproj) {
    let (mut t0, mut t1, mut t2, mut t3, mut t4, mut t5, mut t6) = ([[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2]);

    fp2sqr1271(p.z, &mut t4);                          // t4 = Z1^2
    fp2mul1271(p.y, p.z, &mut t3);                    // t3 = Y1*Z1
    fp2mul1271(t4, f2elm_from_array(C_PHI_4), &mut t0);           // t0 = cphi4*t4
    fp2sqr1271(p.y, &mut t2);                          // t2 = Y1^2
    fp2add1271(t0, t2, &mut t0);                        // t0 = t0+t2
    fp2mul1271(t3, f2elm_from_array(C_PHI_3), &mut t1);           // t1 = cphi3*t3
    fp2sub1271(t0, t1, &mut t5);                        // t5 = t0-t1
    fp2add1271(t0, t1, &mut t0);                        // t0 = t0+t1
    fp2mul1271(t0, p.z, &mut t0);                      // t0 = t0*Z1
    fp2mul1271(t3, f2elm_from_array(C_PHI_1), &mut t1);           // t1 = cphi1*t3
    fp2mul1271(t0, t5, &mut t0);                        // t0 = t0*t5
    fp2mul1271(t4, f2elm_from_array(C_PHI_2), &mut t5);           // t5 = cphi2*t4
    fp2add1271(t2, t5, &mut t5);                        // t5 = t2+t5
    fp2sub1271(t1, t5, &mut t6);                        // t6 = t1-t5
    fp2add1271(t1, t5, &mut t1);                        // t1 = t1+t5
    fp2mul1271(t6, t1, &mut t6);                        // t6 = t1*t6
    fp2mul1271(t6, f2elm_from_array(C_PHI_0), &mut t6);           // t6 = cphi0*t6
    fp2mul1271(p.x, t6, &mut p.x);                    // X = X1*t6
    fp2sqr1271(t2, &mut t6);                            // t6 = t2^2
    fp2sqr1271(t3, &mut t2);                            // t2 = t3^2
    fp2sqr1271(t4, &mut t3);                            // t3 = t4^2
    fp2mul1271(t2, f2elm_from_array(C_PHI_8), &mut t1);           // t1 = cphi8*t2
    fp2mul1271(t3, f2elm_from_array(C_PHI_9), &mut t5);           // t5 = cphi9*t3
    fp2add1271(t1, t6, &mut t1);                        // t1 = t1+t6
    fp2mul1271(t2, f2elm_from_array(C_PHI_6), &mut t2);           // t2 = cphi6*t2
    fp2mul1271(t3, f2elm_from_array(C_PHI_7), &mut t3);           // t3 = cphi7*t3
    fp2add1271(t1, t5, &mut t1);                        // t1 = t1+t5
    fp2add1271(t2, t3, &mut t2);                        // t2 = t2+t3
    fp2mul1271(t1, p.y, &mut t1);                      // t1 = Y1*t1
    fp2add1271(t6, t2, &mut p.y);                      // Y = t6+t2
    fp2mul1271(p.x, t1, &mut p.x);                    // X = X*t1
    fp2mul1271(p.y, f2elm_from_array(C_PHI_5), &mut p.y);       // Y = cphi5*Y
    fpneg1271(&mut p.x[1]);                            // Xfinal = X^p
    fp2mul1271(p.y, p.z, &mut p.y);                  // Y = Y*Z1
    fp2mul1271(t0, t1, &mut p.z);                      // Z = t0*t1
    fp2mul1271(p.y, t0, &mut p.y);                    // Y = Y*t0
    fpneg1271(&mut p.z[1]);                            // Zfinal = Z^p
    fpneg1271(&mut p.y[1]);                            // Yfinal = Y^p
}


/// Apply delta_psi_delta mapping to a point, P = delta(psi_W(delta_inv(P))), 
/// where psi_W is the endomorphism on the Weierstrass form
#[inline]
pub fn ecc_delpsidel(p: &mut PointExtproj) {
    let (mut t0, mut t1, mut t2) = ([[0u64; 2]; 2], [[0u64; 2]; 2], [[0u64; 2]; 2]);

    fpneg1271(&mut p.x[1]);                            // X = X1^p
    fpneg1271(&mut p.z[1]);                            // Z = Z1^p
    fpneg1271(&mut p.y[1]);                            // Y = Y1^p
    fp2sqr1271(p.z, &mut t2);                          // t2 = Z1^p^2
    fp2sqr1271(p.x, &mut t0);                          // t0 = X1^p^2
    fp2mul1271(p.x, t2, &mut p.x);                    // X = X1^p*Z1^p^2
    fp2mul1271(t2, f2elm_from_array(C_PSI_2), &mut p.z);         // Z = cpsi2*Z1^p^2
    fp2mul1271(t2, f2elm_from_array(C_PSI_3), &mut t1);           // t1 = cpsi3*Z1^p^2
    fp2mul1271(t2, f2elm_from_array(C_PSI_4), &mut t2);           // t2 = cpsi4*Z1^p^2
    fp2add1271(t0, p.z, &mut p.z);                    // Z = X1^p^2 + cpsi2*Z1^p^2
    fp2add1271(t0, t2, &mut t2);                        // t2 = X1^p^2 + cpsi4*Z1^p^2
    fp2add1271(t0, t1, &mut t1);                        // t1 = X1^p^2 + cpsi3*Z1^p^2
    fp2neg1271(&mut t2);                                // t2 = -(X1^p^2 + cpsi4*Z1^p^2)
    fp2mul1271(p.z, p.y, &mut p.z);                  // Z = Y1^p*(X1^p^2 + cpsi2*Z1^p^2)
    fp2mul1271(p.x, t2, &mut p.x);                    // X = -X1^p*Z1^p^2*(X1^p^2 + cpsi4*Z1^p^2)
    fp2mul1271(t1, p.z, &mut p.y);                    // Yfinal = t1*Z
    fp2mul1271(p.x, f2elm_from_array(C_PSI_1), &mut p.x);       // Xfinal = cpsi1*X
    fp2mul1271(p.z, t2, &mut p.z);                    // Zfinal = Z*t2
}

/// Apply psi mapping to a point, P = psi(P)
#[inline]
pub fn ecc_psi(p: &mut PointExtproj) {
    ecc_tau(p);
    ecc_delpsidel(p);
    ecc_tau_dual(p);
}

#[inline]
pub fn ecc_phi(p: &mut PointExtproj) {
    ecc_tau(p);
    ecc_delphidel(p);
    ecc_tau_dual(p);
}

#[inline]
pub fn eccneg_extproj_precomp(p: &PointExtprojPrecomp, q: &mut PointExtprojPrecomp) {
    q.t2.copy_from_slice(&p.t2);
    q.yx.copy_from_slice(&p.xy);
    q.xy.copy_from_slice(&p.yx);
    q.z2.copy_from_slice(&p.z2);
    fp2neg1271(&mut q.t2);
}

#[inline]
pub fn eccneg_precomp(p: &PointPrecomp, q: &mut PointPrecomp) {
    q.t2 = p.t2;
    q.yx = p.xy;
    q.xy = p.yx;
    fp2neg1271(&mut q.t2);
}

#[inline]
pub fn mul_truncate(s: &[u64], c: &[u64]) -> u64 {
    let (mut t0, mut t1, mut t2, mut t3) = (0u64, 0u64, 0u64, 0u64);
    let (mut t4, mut t5, mut t6, mut t7) = (0u64, 0u64, 0u64, 0u64);
    let (mut t8, mut t9, mut t10, mut t11) = (0u64, 0u64, 0u64, 0u64);
    let (mut t12, mut t13, mut t14, mut t15) = (0u64, 0u64, 0u64, 0u64);
    let mut t16 = 0u64;

    let (mut high00, mut low10, mut high10, mut low01) = (0u64, 0u64, 0u64, 0u64);
    let (mut high01, mut low20, mut high20, mut low02) = (0u64, 0u64, 0u64, 0u64);
    let (mut high02, mut low11, mut high11, mut low03) = (0u64, 0u64, 0u64, 0u64);
    let (mut high03, mut low30, mut high30, mut low12) = (0u64, 0u64, 0u64, 0u64);
    let mut high12 = 0u64;
    let mut high21 = 0u64;

    _umul128(s[0], c[0], &mut high00);
    low10 = _umul128(s[1], c[0], &mut high10);
    addcarry_u64(addcarry_u64(0, high00, low10, &mut t0), high10, 0, &mut t1);
    low01 = _umul128(s[0], c[1], &mut high01);
    t2 = addcarry_u64(addcarry_u64(0, t0, low01, &mut t0), t1, high01, &mut t3) as u64;
    low20 = _umul128(s[2], c[0], &mut high20) as u64;
    addcarry_u64(addcarry_u64(0, t3, low20, &mut t4), t2, high20, &mut t5);
    low02 = _umul128(s[0], c[2], &mut high02);
    t6 = addcarry_u64(addcarry_u64(0, t4, low02, &mut t7), t5, high02, &mut t8) as u64;
    low11 = _umul128(s[1], c[1], &mut high11);
    t9 = addcarry_u64(addcarry_u64(0, t7, low11, &mut t0), t8, high11, &mut t10) as u64;
    low03 = _umul128(s[0], c[3], &mut high03);
    addcarry_u64(addcarry_u64(0, t10, low03, &mut t11), t6 + t9, high03, &mut t12);
    low30 = _umul128(s[3], c[0], &mut high30);
    addcarry_u64(addcarry_u64(0, t11, low30, &mut t13), t12, high30, &mut t14);
    low12 = _umul128(s[1], c[2], &mut high12);
    addcarry_u64(addcarry_u64(0, t13, low12, &mut t15), t14, high12, &mut t16);

    addcarry_u64(0, t15, _umul128(s[2], c[1], &mut high21), &mut t0) as u64 + t16 + high21 + s[1] * c[3] + s[2] * c[2] + s[3] * c[1]
}


/// Scalar decomposition for the variable-base scalar multiplication
#[inline]
pub fn decompose(k: &[u64], scalars: &mut [u64]) {
    let a1 = mul_truncate(k, &ELL_1);
    let a2 = mul_truncate(k, &ELL_2);
    let a3 = mul_truncate(k, &ELL_3);
    let a4 = mul_truncate(k, &ELL_4);

    scalars[0] = a1 * B11 + a2 * B21 + a3 * B31 + a4 * B41 + C1 + k[0];
    scalars[1] = a1 * B12 + a2 * B22 + a3 * B32 + a4 * B42 + C2;
    scalars[2] = a1 * B13 + a2 * B23 + a3 * B33 + a4 * B43 + C3;
    scalars[3] = a1 * B14 + a2 * B24 + a3 * B34 + a4 * B44 + C4;

    if scalars[0] & 1 == 0 {
        scalars[0] -= B41;
        scalars[1] -= B42;
        scalars[2] -= B43;
        scalars[3] -= B44;
    }
}

/// Computes wNAF recoding of a scalar, where digits are in set {0,+-1,+-3,...,+-(2^(w-1)-1)}
#[inline]
pub fn w_naf_recode(mut scalar: u64, w: u64, digits: &mut [i8]) {
    let val1 = (1 << (w - 1)) - 1;
    let val2 = 1 << w;

    let mask = val2 as u64 - 1;
    let mut index = 0;

    while scalar != 0 {
        let mut digit = (scalar & 1) as i32;

        if digit == 0 {
            scalar >>= 1;
            digits[index] = 0;
        } else {
            digit = (scalar & mask) as i32;
            scalar >>= w;

            if digit > val1 {
                digit -= val2;
            }

            if digit < 0 {
                scalar += 1;
            }

            digits[index] = digit as i8;
            if scalar != 0 {
                for _ in 0..(w-1) {
                    index += 1;
                    digits[index] = 0;
                }
            }
        }

        index += 1;
    }
}

/// Generation of the precomputation table used internally by the double scalar multiplication function ecc_mul_double()
#[inline]
pub fn ecc_precomp_double(p: &mut PointExtproj, table: &mut [PointExtprojPrecomp]) {
    let mut q = PointExtproj::default();
    let mut pp = PointExtprojPrecomp::default();

    r1_to_r2(p, &mut table[0]);
    eccdouble(p);
    r1_to_r3(&p, &mut pp);

    eccadd_core(&table[0], &pp, &mut q);
    r1_to_r2(&q, &mut table[1]);

    eccadd_core(&table[1], &pp, &mut q);
    r1_to_r2(&q, &mut table[2]);

    eccadd_core(&table[2], &pp, &mut q);
    r1_to_r2(&q, &mut table[3]);
}

/// Double scalar multiplication R = k*G + l*Q, where the G is the generator
/// Uses DOUBLE_SCALAR_TABLE, which contains multiples of G, Phi(G), Psi(G) and Phi(Psi(G))
/// The function uses wNAF with interleaving.
#[inline]
pub fn ecc_mul_double(k: &mut [u64], l: &mut [u64], q: &mut PointAffine) -> bool {
    let (mut digits_k1, mut digits_k2, mut digits_k3, mut digits_k4) = ([0i8; 65], [0i8; 65], [0i8; 65], [0i8; 65]);
    let (mut digits_l1, mut digits_l2, mut digits_l3, mut digits_l4) = ([0i8; 65], [0i8; 65], [0i8; 65], [0i8; 65]);
    let mut v = PointPrecomp::default();
    let (mut q1, mut q2, mut q3, mut q4, mut t) = (PointExtproj::default(), PointExtproj::default(), PointExtproj::default(), PointExtproj::default(), PointExtproj::default());
    let mut u = PointExtprojPrecomp::default();
    let (mut q_table1, mut q_table2, mut q_table3, mut q_table4) = ([PointExtprojPrecomp::default(); 4], [PointExtprojPrecomp::default(); 4], [PointExtprojPrecomp::default(); 4], [PointExtprojPrecomp::default(); 4]);
    let mut k_scalars = [0u64; 4];
    let mut l_scalars = [0u64; 4];

    point_setup(&q, &mut q1);

    if !ecc_point_validate(&q1) {
        return false;
    }

    q2 = q1;
    ecc_phi(&mut q2);
    q3 = q1;
    ecc_psi(&mut q3);
    q4 = q2;
    ecc_psi(&mut q4);

    decompose(k, &mut k_scalars);
    decompose(l, &mut l_scalars);
    w_naf_recode(k_scalars[0], 8, &mut digits_k1);                        // Scalar recoding
    w_naf_recode(k_scalars[1], 8, &mut digits_k2);
    w_naf_recode(k_scalars[2], 8, &mut digits_k3);
    w_naf_recode(k_scalars[3], 8, &mut digits_k4);
    w_naf_recode(l_scalars[0], 4, &mut digits_l1);
    w_naf_recode(l_scalars[1], 4, &mut digits_l2);
    w_naf_recode(l_scalars[2], 4, &mut digits_l3);
    w_naf_recode(l_scalars[3], 4, &mut digits_l4);

    ecc_precomp_double(&mut q1, &mut q_table1);
    ecc_precomp_double(&mut q2, &mut q_table2);
    ecc_precomp_double(&mut q3, &mut q_table3);
    ecc_precomp_double(&mut q4, &mut q_table4);

    t.x[0][0] = 0; t.x[0][1] = 0; t.x[1][0] = 0; t.x[1][1] = 0; // Initialize T as the neutral point (0:1:1)
    t.y[0][0] = 1; t.y[0][1] = 0; t.y[1][0] = 0; t.y[1][1] = 0;
    t.z[0][0] = 1; t.z[0][1] = 0; t.z[1][0] = 0; t.z[1][1] = 0;

    for i in (0..64).rev() {
        eccdouble(&mut t);

        if digits_l1[i] < 0 {
            eccneg_extproj_precomp(&q_table1[((-digits_l1[i]) >> 1) as usize], &mut u);
            eccadd(&u, &mut t);
        }
        else if digits_l1[i] > 0 {
            eccadd(&q_table1[((digits_l1[i]) >> 1) as usize], &mut t);
        }

        if digits_l2[i] < 0 {
            eccneg_extproj_precomp(&q_table2[((-digits_l2[i]) >> 1) as usize], &mut u);
            eccadd(&u, &mut t);
        }
        else if digits_l2[i] > 0 {
            eccadd(&q_table2[((digits_l2[i]) >> 1) as usize], &mut t);
        }

        if digits_l3[i] < 0 {
            eccneg_extproj_precomp(&q_table3[((-digits_l3[i]) >> 1) as usize], &mut u);
            eccadd(&u, &mut t);
        }
        else if digits_l3[i] > 0 {
            eccadd(&q_table3[((digits_l3[i]) >> 1) as usize], &mut t);
        }

        if digits_l4[i] < 0 {
            eccneg_extproj_precomp(&q_table4[((-digits_l4[i]) >> 1) as usize], &mut u);
            eccadd(&u, &mut t);
        }
        else if digits_l4[i] > 0 {
            eccadd(&q_table4[((digits_l4[i]) >> 1) as usize], &mut t);
        }


        unsafe {

            if digits_k1[i] < 0 {
                eccneg_precomp(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(((-digits_k1[i]) >> 1) as isize), &mut v);
                eccmadd(&v, &mut t);
            } else if digits_k1[i] > 0 {
                eccmadd(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(((digits_k1[i]) >> 1) as isize), &mut t);
            }

            if digits_k2[i] < 0 {
                eccneg_precomp(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(64 + ((-digits_k2[i]) >> 1) as isize), &mut v);
                eccmadd(&v, &mut t);
            } else if digits_k2[i] > 0 {
                eccmadd(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(64 + ((digits_k2[i]) >> 1) as isize), &mut t);
            }

            if digits_k3[i] < 0 {
                eccneg_precomp(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(2 * 64 + ((-digits_k3[i]) >> 1) as isize), &mut v);
                eccmadd(&v, &mut t);
            } else if digits_k3[i] > 0 {
                eccmadd(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(2* 64 + ((digits_k3[i]) >> 1) as isize), &mut t);
            }

            if digits_k4[i] < 0 {
                eccneg_precomp(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(3 * 64 + ((-digits_k4[i]) >> 1) as isize), &mut v);
                eccmadd(&v, &mut t);
            } else if digits_k4[i] > 0 {
                eccmadd(&*(DOUBLE_SCALAR_TABLE.as_ptr() as *const PointPrecomp).offset(3 * 64 + ((digits_k4[i]) >> 1) as isize ), &mut t);
            }
        }
    }

    eccnorm(&mut t, q);

    true
}

/// Generation of the precomputation table used by the variable-base scalar multiplication ecc_mul()
#[inline]
pub fn ecc_precomp(p: &mut PointExtproj, t: &mut [PointExtprojPrecomp]) {
    let (mut q, mut r, mut s) = (PointExtprojPrecomp::default(), PointExtprojPrecomp::default(), PointExtprojPrecomp::default());
    let mut pp = *p;

    ecc_phi(&mut pp);
    r1_to_r3(&pp, &mut q);

    ecc_psi(&mut pp);
    r1_to_r3(&pp, &mut s);

    r1_to_r2(&p, &mut t[0]);

    ecc_psi(p);
    r1_to_r3(&p, &mut r);

    eccadd_core(&t[0], &q, &mut pp);              // T[1] = P+Q using the representations (X,Y,Z,Ta,Tb) <- (X+Y,Y-X,2Z,2dT) + (X+Y,Y-X,Z,T)
    r1_to_r2(&pp, &mut t[1]);                    // Converting from (X,Y,Z,Ta,Tb) to (X+Y,Y-X,2Z,2dT)
    eccadd_core(&t[0], &r, &mut pp);              // T[2] = P+r 
    r1_to_r2(&pp, &mut t[2]);
    eccadd_core(&t[1], &r, &mut pp);              // T[3] = P+Q+r 
    r1_to_r2(&pp, &mut t[3]);
    eccadd_core(&t[0], &s, &mut pp);              // T[4] = P+S 
    r1_to_r2(&pp, &mut t[4]);
    eccadd_core(&t[1], &s, &mut pp);              // T[5] = P+Q+S 
    r1_to_r2(&pp, &mut t[5]);
    eccadd_core(&t[2], &s, &mut pp);              // T[6] = P+r+S 
    r1_to_r2(&pp, &mut t[6]);
    eccadd_core(&t[3], &s, &mut pp);              // T[7] = P+Q+r+S 
    r1_to_r2(&pp, &mut t[7]);
}

/// Co-factor clearing
#[inline]
pub fn cofactor_clearing(r: &mut PointExtproj) {
    let mut q = PointExtprojPrecomp::default();

    r1_to_r2(&r, &mut q);                      // Converting from (X,Y,Z,Ta,Tb) to (X+Y,Y-X,2Z,2dT)
    eccdouble(r);                        // P = 2*P using representations (X,Y,Z,Ta,Tb) <- 2*(X,Y,Z)
    eccadd(&q, r);                        // P = P+Q using representations (X,Y,Z,Ta,Tb) <- (X,Y,Z,Ta,Tb) + (X+Y,Y-X,2Z,2dT)
    eccdouble(r);
    eccdouble(r);
    eccdouble(r);
    eccdouble(r);
    eccadd(&q, r);
    eccdouble(r);
    eccdouble(r);
    eccdouble(r);
}

#[inline]
pub fn ecc_mul(p: &mut PointAffine, k: &[u64], q: &mut PointAffine) -> bool {
    let mut r = PointExtproj::default();
    let mut table = [[PointExtprojPrecomp::default(); 8]; 2];
    let mut scalars = [0u64; 4];
    let mut digits = [0u64; 64];
    let mut sign_masks = [0u64; 64];

    point_setup(&p, &mut r);

    if !ecc_point_validate(&r) {
        return false;
    }

    decompose(k, &mut scalars);

    cofactor_clearing(&mut r);


    for i in 0..64 {
        scalars[0] >>= 1;
        let bit0 = scalars[0] & 1;
        sign_masks[i] = bit0;

        digits[i] = scalars[1] & 1;
        scalars[1] = (scalars[1] >> 1) + ((bit0 | digits[i]) ^ bit0);

        let mut  bit = scalars[2] & 1;
        scalars[2] = (scalars[2] >> 1) + ((bit0 | bit) ^ bit0);
        digits[i] += bit << 1;

        bit = scalars[3] & 1;
        scalars[3] = (scalars[3] >> 1) + ((bit0 | bit) ^ bit0);
        digits[i] += bit << 2;
    }

    ecc_precomp(&mut r, &mut table[1]);

    for i in 0..8 {
        table[0][i].xy = table[1][i].yx;
        table[0][i].yx = table[1][i].xy;
        table[0][i].t2 = table[1][i].t2;
        table[0][i].z2 = table[1][i].z2;
        fp2neg1271(&mut table[0][i].t2);
    }

    r2_to_r4(&table[1][(scalars[1] + (scalars[2] << 1) + (scalars[3] << 2)) as usize], &mut r);


    for i in (0..64).rev() {
        eccdouble(&mut r);
        eccadd(&table[sign_masks[i] as usize][digits[i] as usize], &mut r);
    }
    eccnorm(&mut r, q);

    true
}


/// Encode point P
#[inline]
pub fn encode(p: &mut PointAffine, pencoded: &mut [u8]) {
    let temp1 = (p.x[1][1] & 0x4000000000000000) << 1;
    let temp2 = (p.x[0][1] & 0x4000000000000000) << 1;

    unsafe {
        copy_nonoverlapping(p.y.as_ptr() as *const u8, pencoded.as_mut_ptr(), 32);

        if p.x[0][0] == 0 && p.x[0][1] == 0 {
            let bytes = temp1.to_le_bytes();
            for i in 0..8 {
                pencoded[3*8 + i] |= bytes[i];
            }
        } else {
            let bytes = temp2.to_le_bytes();
            for i in 0..8 {
                pencoded[3*8 + i] |= bytes[i];
            }
        }
    }
}


#[inline]
pub fn decode(pencoded: &[u8], p: &mut PointAffine) -> bool {
    let (mut r, mut t, mut t0, mut t1, mut t2, mut t3, mut t4) = ([0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2]);
    let (mut u, mut v) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);
    let mut r_l = PointExtproj::default();

    unsafe {
        copy_nonoverlapping(pencoded.as_ptr(), p.y.as_mut_ptr() as *mut u8, 32);
        p.y[1][1] &= 0x7FFFFFFFFFFFFFFF;

        fp2sqr1271(p.y, &mut u);
        fp2mul1271(u, f2elm_from_array(PARAMETER_D), &mut v);
        fp2sub1271(u, f2elm_from_array(ONE), &mut u);
        fp2add1271(v, f2elm_from_array(ONE), &mut v);

        fpsqr1271(v[0], &mut t0);                            // t0 = v0^2
        fpsqr1271(v[1], &mut t1);                            // t1 = v1^2
        fpadd1271(t0, t1, &mut t0);                          // t0 = t0+t1   
        fpmul1271(u[0], v[0], &mut t1);                      // t1 = u0*v0
        fpmul1271(u[1], v[1], &mut t2);                      // t2 = u1*v1 
        fpadd1271(t1, t2, &mut t1);                          // t1 = t1+t2  
        fpmul1271(u[1], v[0], &mut t2);                      // t2 = u1*v0
        fpmul1271(u[0], v[1], &mut t3);                      // t3 = u0*v1
        fpsub1271(t2, t3, &mut t2);                          // t2 = t2-t3    
        fpsqr1271(t1, &mut t3);                              // t3 = t1^2    
        fpsqr1271(t2, &mut t4);                              // t4 = t2^2
        fpadd1271(t3, t4, &mut t3);                          // t3 = t3+t4

        for _ in 0..125 {
            fpsqr1271(t3, &mut t3);
        }

        fpadd1271(t1, t3, &mut t);
        mod1271(&mut t);

        if t[0] == 0 && t[1] == 0 {
            fpsub1271(t, t, &mut t);
        }

        fpadd1271(t, t, &mut t);                             // t = 2*t            
        fpsqr1271(t0, &mut t3);                              // t3 = t0^2      
        fpmul1271(t0, t3, &mut t3);                          // t3 = t3*t0   
        fpmul1271(t, t3, &mut t3);                           // t3 = t3*t
        fpexp1251(t3, &mut r);                               // r = t3^(2^125-1)  
        fpmul1271(t0, r, &mut t3);                           // t3 = t0*r          
        fpmul1271(t, t3, &mut p.x[0]);                      // x0 = t*t3 
        fpsqr1271(p.x[0], &mut t1);
        fpmul1271(t0, t1, &mut t1);                          // t1 = t0*x0^2

        let mut temp = [0u64; 2];
        let mask = 0 - (1 & p.x[0][0]);
        addcarry_u64(addcarry_u64(0, p.x[0][0], mask, &mut temp[0]), p.x[0][1], mask >> 1, &mut temp[1]);
        p.x[0][0] = __shiftright128(temp[0], temp[1], 1);
        p.x[0][1] = temp[1] >> 1;

        fpmul1271(t2, t3, &mut p.x[1]);

        fpsub1271(t, t1, &mut t);
        mod1271(&mut t);

        if t[0] != 0 || t[1] != 0 {
            t0[0] = p.x[0][0];
            t0[1] = p.x[0][1];
            p.x[0][0] = p.x[1][0];
            p.x[0][1] = p.x[1][1];
            p.x[1][0] = t0[0];
            p.x[1][1] = t0[1];
        }

        mod1271(&mut p.x[0]);
        if pencoded[31] >> 7 != (p.x[if p.x[0][0] == 0 && p.x[0][1] == 0 { 1 } else { 0 }][1] >> 62) as u8 {
            fp2neg1271(&mut p.x);
        }

        /*let (a, b) = (p.x[0], p.x[1]);
        p.x[1] = a;
        p.x[0] = b;*/

        point_setup(&p, &mut r_l);

        if !ecc_point_validate(&r_l) {
            fpneg1271(&mut r_l.x[1]);
            p.x[1][0] = r_l.x[1][0];
            p.x[1][1] = r_l.x[1][1];

            if !ecc_point_validate(&r_l) {
                return false;
            }
        }
    }
    
    true
}