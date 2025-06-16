use core::fmt::{Debug, Display};


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct QubicTime {
    pub milliseconds: u16,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8
}

impl Debug for QubicTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2}", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.second))
    }
}

impl Display for QubicTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2}", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.second))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct QubicSetUtcTime {
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    _pad: u8,
    pub nanosecond: u32
}

// non leap years
const YEAR: u128 = 3600*24*365;
const DAY: u128 = 3600*24;
const DAYS_IN_MONTHS: [u128; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
#[cfg(feature = "std")]
impl From<std::time::SystemTime> for QubicSetUtcTime {
    fn from(time: std::time::SystemTime) -> Self {
        let unix_epoch = time.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();

        let qubic_epoch = unix_epoch/1_000_000_000 - 30*YEAR - (30/4)*3600*24;
        let year = qubic_epoch/YEAR;
        let is_leap = year%4 == 0;
        let mut day = qubic_epoch/DAY - (365*year + (year/4 - is_leap as u8 as u128));

        let mut month = 0;
        for i in 0..12 {
            let days_in_month = DAYS_IN_MONTHS[i] + (if i == 1 && is_leap { 1 } else { 0 });
            if day < days_in_month {
                month = i + 1;
                break;
            }

            day -= days_in_month;
        }

        let hour = (qubic_epoch%DAY)/3600;
        let minute = (qubic_epoch%3600)/60;
        let second = qubic_epoch%60;
        let nanosecond = unix_epoch%1_000_000_000;
        
        Self { year: year as u8, month: month as u8, day: day as u8, hour: hour as u8, minute: minute as u8, second: second as u8, _pad: 0, nanosecond: nanosecond as u32 }
    }
}

impl Debug for QubicSetUtcTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2}", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.second))
    }
}

impl Display for QubicSetUtcTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2}", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.second))
    }
}

#[test]
fn test_time() {
    let now = std::time::SystemTime::now();

    dbg!(QubicSetUtcTime::from(now));
}