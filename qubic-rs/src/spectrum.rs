use anyhow::Result;
use qubic_rs::qubic_tcp_types::types::Entity;
use qubic_rs::qubic_types::QubicId;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

pub const SPECTRUM_DEPTH: usize = 24;
pub const SPECTRUM_CAPACITY: usize = 0x1000000;

pub struct SpectrumFile {
    spectrum: Vec<Entity>,
    compressed: Vec<Entity>,
}

impl SpectrumFile {
    pub fn load_file(file: &str) -> Result<Self> {
        let path = Path::new(file);

        let mut spectrum = vec![
            Entity {
                public_key: QubicId::default(),
                incoming_amount: 0,
                outgoing_amount: 0,
                number_of_incoming_transfers: 0,
                number_of_outgoing_transfers: 0,
                latest_incoming_transfer_tick: 0,
                latest_outgoing_transfer_tick: 0
            };
            SPECTRUM_CAPACITY
        ];

        let file = File::open(path)?;

        let mut reader = BufReader::new(file);

        unsafe {
            let slice = std::ptr::slice_from_raw_parts_mut(
                spectrum.as_mut_ptr() as *mut u8,
                SPECTRUM_CAPACITY * std::mem::size_of::<Entity>(),
            );
            reader.read_exact(&mut *slice)?;
        }

        let mut compressed: Vec<Entity> = Vec::new();

        for e in spectrum.iter() {
            if e.incoming_amount - e.outgoing_amount != 0 {
                compressed.push(*e);
            }
        }

        compressed.sort_by_key(|e1| (e1.balance()));

        Ok(Self {
            spectrum,
            compressed,
        })
    }

    pub fn get_spectrum_index(&self, public_key: &QubicId) -> Option<usize> {
        if public_key.0 == QubicId::default().0 {
            return None;
        }

        let mut index = u32::from_le_bytes(public_key.0[..4].try_into().unwrap()) as usize
            & (SPECTRUM_CAPACITY - 1);

        loop {
            if self.spectrum[index].public_key.0 == public_key.0 {
                return Some(index);
            }

            if self.spectrum[index].public_key.0 == QubicId::default().0 {
                return None;
            } else {
                index = (index + 1) & (SPECTRUM_CAPACITY - 1);
            }
        }
    }

    pub fn get_energy(&self, index: usize) -> u64 {
        self.spectrum[index].incoming_amount - self.spectrum[index].outgoing_amount
    }

    pub fn increase_energy(&mut self, public_key: &QubicId, amount: u64, tick: u32) {
        if public_key.0 != QubicId::default().0 {
            let mut index = u32::from_le_bytes(public_key.0[..4].try_into().unwrap()) as usize
                & (SPECTRUM_CAPACITY - 1);

            loop {
                if self.spectrum[index].public_key.0 == public_key.0 {
                    self.spectrum[index].incoming_amount += amount;
                    self.spectrum[index].number_of_incoming_transfers += 1;
                    self.spectrum[index].latest_incoming_transfer_tick = tick;
                    break;
                }

                if self.spectrum[index].public_key.0 == QubicId::default().0 {
                    self.spectrum[index].public_key = *public_key;
                    self.spectrum[index].incoming_amount = amount;
                    self.spectrum[index].number_of_incoming_transfers = 1;
                    self.spectrum[index].latest_incoming_transfer_tick = tick;
                } else {
                    index = (index + 1) & (SPECTRUM_CAPACITY - 1);
                }
            }
        }
    }

    pub fn decrease_energy(&mut self, index: usize, amount: u64, tick: u32) -> bool {
        if self.get_energy(index) >= amount {
            self.spectrum[index].outgoing_amount += amount;
            self.spectrum[index].number_of_outgoing_transfers += 1;
            self.spectrum[index].latest_outgoing_transfer_tick = tick;

            return true;
        }

        false
    }

    pub fn get_supply(&self) -> u64 {
        let mut bal = 0;
        for e in self.compressed.iter() {
            bal += e.balance();
        }

        bal
    }

    pub fn get_top_holders(&self, to: usize) -> Vec<Entity> {
        let mut th = self.compressed[self.compressed.len() - to..self.compressed.len()].to_vec();

        th.reverse();

        th
    }

    pub fn get_amount_min(&self, cmp: u64) -> usize {
        let mut wallets = 0;

        for e in self.compressed.iter() {
            if e.balance() > cmp {
                wallets += 1;
            }
        }

        wallets
    }
}

#[test]
fn test() {
    use std::str::FromStr;
    let spectrum = SpectrumFile::load_file("spectrum/spectrum.090").unwrap();

    let index = spectrum.get_spectrum_index(
        &QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA").unwrap(),
    );

    dbg!(spectrum.get_supply() / 1_000_000_000);
    let th = spectrum.get_top_holders(10);
    dbg!(th.iter().map(|e| e.balance()).sum::<u64>() / 1_000_000_000);
    dbg!(spectrum.get_amount_min(1_000_000_000));

    if let Some(index) = index {
        dbg!(spectrum.spectrum[index]);
    }
}