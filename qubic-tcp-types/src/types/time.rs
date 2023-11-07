use std::fmt::{Debug, Display};


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct QubicTime {
    pub milliseconds: u16,
    pub seconds: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8
}

impl Debug for QubicTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2}", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.seconds))
    }
}

impl Display for QubicTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2}", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.seconds))
    }
}