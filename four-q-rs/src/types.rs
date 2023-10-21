/// Datatype for representing 128-bit field elements
pub type FelmT = [u64; 2]; 

/// Datatype for representing quadratic extension field elements
pub type F2elmT = [FelmT; 2]; 

/// Point representation in affine coordinates
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct PointAffine { 
    pub x: F2elmT,
    pub y: F2elmT
}


#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct PointExtproj { 
    pub x: F2elmT,
    pub y: F2elmT,
    pub z: F2elmT,
    pub ta: F2elmT,
    pub tb: F2elmT
}


/// Point representation in extended coordinates (for precomputed points)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct PointExtprojPrecomp { 
    pub xy: F2elmT,
    pub yx: F2elmT,
    pub z2: F2elmT,
    pub t2: F2elmT
}

/// Point representation in extended affine coordinates (for precomputed points)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct PointPrecomp { 
    pub xy: F2elmT,
    pub yx: F2elmT,
    pub t2: F2elmT
}

pub type PointPrecompT = [PointPrecomp; 1];