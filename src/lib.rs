use chrono::{Date, DateTime, Duration, Local, TimeZone};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
extern crate chrono;

const MAGIC_NUMBER: [u8; 3] = [0xE0, 0xC5, 0xEA];
const END_OF_DATA: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

pub struct VoltcraftData {
    raw_data: Vec<u8>,
}

#[derive(Debug)]
pub struct PowerItem {
    pub timestamp: chrono::DateTime<Local>, // timestamp
    pub voltage: f64,                       // volts
    pub current: f64,                       // ampers
    pub power_factor: f64,                  // cos(phi)
    pub power: f64,                         //kW
    pub apparent_power: f64,                //kVA
}

impl VoltcraftData {
    pub fn from_file(filename: &str) -> Result<VoltcraftData, &'static str> {
        let contents = fs::read(filename);
        match contents {
            Err(_) => return Err("Error reading file"),
            Ok(raw_data) => return Ok(VoltcraftData { raw_data }),
        };
    }

    pub fn from_raw(raw_data: Vec<u8>) -> VoltcraftData {
        VoltcraftData { raw_data }
    }

    pub fn parse(&self) -> Result<Vec<PowerItem>, &'static str> {
        // Make sure we parse valid Voltcraft data
        if !self.is_valid() {
            return Err("Invalid data (not a Voltcraft file)");
        }

        // The data starts after the magic number
        let mut offset = MAGIC_NUMBER.len();
        // Decode the starting timestamp of the data.
        // Each power item is recorded at 1 minute intervals, so we will increment the time accordingly.
        let start_time = self.decode_timestamp(offset);
        let mut minute_increment = 0;
        offset += 5;
        // Decode power items until "end of data" (#FF FF FF FF) is encountered
        let mut result = Vec::<PowerItem>::new();
        loop {
            if self.is_endofdata(offset) {
                break;
            }
            let power_data = self.decode_power(offset);
            let power_timestamp = start_time + Duration::minutes(minute_increment);
            minute_increment += 1; // increment time offset
            offset += 5; // increment byte offset
            result.push(PowerItem {
                timestamp: power_timestamp,
                voltage: power_data.0,
                current: power_data.1,
                power_factor: power_data.2,
                power: power_data.3,
                apparent_power: power_data.4,
            });
        }
        Ok(result)
    }

    fn is_valid(&self) -> bool {
        let header = &self.raw_data[0..3];
        header == MAGIC_NUMBER
    }

    fn is_endofdata(&self, off: usize) -> bool {
        let eod = &self.raw_data[off..off + 4];
        eod == END_OF_DATA
    }

    fn decode_timestamp(&self, off: usize) -> chrono::DateTime<Local> {
        let month: u8 = self.raw_data[off + 0].into();
        let day: u8 = self.raw_data[off + 1].into();
        let year: u8 = self.raw_data[off + 2].into();
        let hour: u8 = self.raw_data[off + 3].into();
        let minute: u8 = self.raw_data[off + 4].into();
        chrono::Local
            .ymd(year as i32 + 2000, month as u32, day as u32)
            .and_hms(hour as u32, minute as u32, 0)
    }

    fn decode_power(&self, off: usize) -> (f64, f64, f64, f64, f64) {
        // Decode voltage (2 bytes - Big Endian)
        let voltage: [u8; 2] = self.raw_data[off..off + 2].try_into().unwrap();
        let voltage = u16::from_be_bytes(voltage);
        let voltage: f64 = voltage as f64 / 10.0; // volts

        // Decode current (2 bytes - Big Endian)
        let current: [u8; 2] = self.raw_data[off + 2..off + 4].try_into().unwrap();
        let current = u16::from_be_bytes(current);
        let current: f64 = current as f64 / 1000.0; // ampers

        // Decode power factor (1 byte)
        let power_factor: u8 = self.raw_data[off + 4].into();
        let power_factor: f64 = power_factor as f64 / 100.0; // cos phi

        let power = voltage * current * power_factor / 1000.0; // kW
        let apparent_power = voltage * current / 1000.0; // kVA
        (voltage, current, power_factor, power, apparent_power)
    }
}

pub struct VoltcraftStatistics {
    power_data: Vec<PowerItem>,
}

impl VoltcraftStatistics {
    //TODO: - Each day: Total power consumption, min and max voltage, average voltage, max power, average power
    //TODO: - All days: Total power consumption and day average, min and max voltage, max power
    pub fn new(power_data: Vec<PowerItem>) -> VoltcraftStatistics {
        VoltcraftStatistics { power_data }
    }

    pub fn distinct_days(&self) -> Vec<Date<Local>> {
        let mut days = self
            .power_data
            .iter()
            .map(|d| d.timestamp.date())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        days.sort();
        days
    }

    pub fn filter_power_data(&self, day: &Date<Local>) -> Vec<&PowerItem> {
        let filtered_data = self
            .power_data
            .iter()
            .filter(|d| *day == d.timestamp.date())
            .collect::<Vec<_>>();
        filtered_data
    }

    pub fn voltage_minmax(&self) -> (f64, f64) {
        let min_voltage = self
            .power_data
            .iter()
            .min_by(|a, b| a.voltage.partial_cmp(&b.voltage).unwrap())
            .unwrap()
            .voltage;
        let max_voltage = self
            .power_data
            .iter()
            .max_by(|a, b| a.voltage.partial_cmp(&b.voltage).unwrap())
            .unwrap()
            .voltage;

        (min_voltage, max_voltage)
    }
}

#[cfg(test)]

const TESTDATA: [u8; 17] = [
    // Header (magic number)
    0xE0, 0xC5, 0xEA, // Power data
    0x09, 0x0B, 0x0E, 0x12, 0x2B, 0x08, 0xC6, 0x01, 0xBE, 0x57, // End of power data
    0xFF, 0xFF, 0xFF, 0xFF,
];

mod tests {
    use super::*;
    #[test]
    fn voltcraft_valid_data() {
        let vd = VoltcraftData::from_raw(TESTDATA.to_vec());
        assert!(vd.is_valid());
    }

    #[test]
    fn voltcraft_timestamp() {
        let vd = VoltcraftData::from_raw(TESTDATA.to_vec());
        let offset_timestamp = 3;
        let ts = vd.decode_timestamp(offset_timestamp);
        let expected = DateTime::parse_from_rfc3339("2014-09-11T18:43:00+03:00").unwrap();
        assert_eq!(ts, expected);
    }

    #[test]
    fn voltcraft_poweritem() {
        let vd = VoltcraftData::from_raw(TESTDATA.to_vec());
        let offset_poweritem = 8;
        let pw = vd.decode_power(offset_poweritem);
        assert_eq!(pw.0, 224.6);
        assert_eq!(pw.1, 0.446);
        assert_eq!(pw.2, 0.87);
    }

    #[test]
    fn voltcraft_voltage_minmax() {
        let vd = VoltcraftData::from_raw(TESTDATA.to_vec());
        let parsed_data = vd.parse().unwrap();
        let stats = VoltcraftStatistics::new(parsed_data);
        let voltage_stats = stats.voltage_minmax();
        assert_eq!(voltage_stats.0, 224.6);
        assert_eq!(voltage_stats.1, 224.6);
    }
}
