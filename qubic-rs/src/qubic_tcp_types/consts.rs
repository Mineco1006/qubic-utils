use crate::qubic_types::QubicId;

pub const NUMBER_OF_TRANSACTION_PER_TICK: usize = 1024;
pub const MAX_NUMBER_OF_CONTRACTS: usize = 1024;
pub const NUMBER_OF_COMPUTORS: usize = 676;
pub const SPECTRUM_DEPTH: usize = 24;
pub const SPECTRUM_CAPACITY: usize = 0x1000000;
pub const ARBITRATOR: QubicId = QubicId([
    158, 26, 16, 12, 251, 85, 109, 239, 123, 204, 98, 82, 228, 125, 223, 9, 133, 66, 134, 55, 195,
    209, 179, 202, 161, 111, 51, 253, 152, 67, 141, 148,
]);