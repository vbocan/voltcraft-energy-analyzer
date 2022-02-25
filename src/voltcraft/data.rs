use chrono::{Duration, Local, TimeZone};
use std::fs;
pub struct VoltcraftData {
    raw_data: Vec<u8>,
}

#[derive(Debug, Copy, Clone)]
pub struct PowerEvent {
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
            Err(_) => return Err("File not found"),
            Ok(raw_data) => return Ok(VoltcraftData { raw_data }),
        };
    }

    pub fn from_raw(raw_data: Vec<u8>) -> VoltcraftData {
        VoltcraftData { raw_data }
    }

    pub fn parse(&self) -> Result<Vec<PowerEvent>, &'static str> {
        let mut result = Vec::<PowerEvent>::new();
        // The initial offset in the data block is zero
        let mut offset = 0;
        // Set the initial time somewhere in the past as it will be overwritten anyway
        let mut start_time = chrono::Local.ymd(2000, 1, 1).and_hms(0, 0, 0);
        // For each new power event we encounter, the timestamp is increased by one minute (the Voltcraft device records parameters each minute)
        let mut minute_increment = 0;

        // Check whether we have a valid data file (the data block header should be at the beginning of the file)
        if !self.is_datablock(offset) {
            return Err("Invalid data file, probably not a Voltcraft file");
        }

        loop {
            // If we encounter the beginning of a data block, decode and memorize the timestamp
            if self.is_datablock(offset) {
                offset += 3;
                start_time = self.decode_timestamp(offset);
                minute_increment = 0;
                offset += 5;
                continue;
            }
            // Check whether we have reached the end of the Voltcraft data file
            if self.is_endofdata(offset) {
                break;
            }
            let power_data = self.decode_power(offset);
            let power_timestamp = start_time + Duration::minutes(minute_increment);
            minute_increment += 1; // Increment the timestamp by 1 minute
            offset += 5; // Increment byte offset

            result.push(PowerEvent {
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

    fn is_datablock(&self, off: usize) -> bool {
        const MAGIC_NUMBER: [u8; 3] = [0xE0, 0xC5, 0xEA];
        let header = &self.raw_data[off..off + 3];
        header == MAGIC_NUMBER
    }

    fn is_endofdata(&self, off: usize) -> bool {
        const END_OF_DATA: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
        let eod = &self.raw_data[off..off + 4];
        eod == END_OF_DATA
    }

    fn decode_timestamp(&self, off: usize) -> chrono::DateTime<Local> {
        let month: u8 = self.raw_data[off];
        let day: u8 = self.raw_data[off + 1];
        let year: u8 = self.raw_data[off + 2];
        let hour: u8 = self.raw_data[off + 3];
        let minute: u8 = self.raw_data[off + 4];
        chrono::Local
            .ymd(year as i32 + 2000, month as u32, day as u32)
            .and_hms(hour as u32, minute as u32, 0)
    }

    fn decode_power(&self, off: usize) -> (f64, f64, f64, f64, f64) {
        // Decode voltage (2 bytes - Big Endian)
        let voltage: [u8; 2] = self.raw_data[off..off + 2].try_into().unwrap();
        let voltage = u16::from_be_bytes(voltage);
        let voltage: f64 = voltage as f64 / 10.0; // volts
        assert!(voltage > 150.0, "Tensiune mică mare la offset {}", off);
        assert!(voltage < 250.0, "Tensiune mare mare la offset {}", off);

        // Decode current (2 bytes - Big Endian)
        let current: [u8; 2] = self.raw_data[off + 2..off + 4].try_into().unwrap();
        let current = u16::from_be_bytes(current);
        let current: f64 = current as f64 / 1000.0; // ampers

        // Decode power factor (1 byte)
        let power_factor: u8 = self.raw_data[off + 4];
        let power_factor: f64 = power_factor as f64 / 100.0; // cos phi

        let power = voltage * current * power_factor / 1000.0; // kW
        let apparent_power = voltage * current / 1000.0; // kVA
        (voltage, current, power_factor, power, apparent_power)
    }
}

#[cfg(test)]

mod tests {
    use crate::voltcraft::data::VoltcraftData;
    use chrono::DateTime;
    const TESTDATA: [u8; 17] = [
        // Header (magic number)
        0xE0, 0xC5, 0xEA, // Power data
        0x09, 0x0B, 0x0E, 0x12, 0x2B, 0x08, 0xC6, 0x01, 0xBE, 0x57, // End of power data
        0xFF, 0xFF, 0xFF, 0xFF,
    ];

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
}
