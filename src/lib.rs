pub mod voltcraftdata {
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
            assert!(voltage > 150.0, "Tensiune mică mare la offset {}", off);
            assert!(voltage < 250.0, "Tensiune mare mare la offset {}", off);

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

    #[cfg(test)]

    mod tests {
        use crate::voltcraftdata::VoltcraftData;
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
}

pub mod voltcraftstats {
    use crate::voltcraftdata::PowerEvent;
    use chrono::{Date, DateTime, Duration, Local};
    use itertools::Itertools;
    use std::collections::HashSet;

    pub struct VoltcraftStatistics<'a> {
        power_data: &'a Vec<PowerEvent>,
    }

    #[derive(Debug, Copy, Clone)]
    pub struct PowerStats {
        pub total_active_power: f64,      // total active power (kWh)
        pub avg_active_power: f64,        // average active power (kW)
        pub max_active_power: PowerEvent, // maxiumum active power

        pub total_apparent_power: f64, // total apparent power (kWh)
        pub avg_apparent_power: f64,   // average apparent power (kW)
        pub max_apparent_power: PowerEvent, // maxiumum apparent power

        pub min_voltage: PowerEvent, // minimum voltage
        pub max_voltage: PowerEvent, // maximum voltage
        pub avg_voltage: f64,        // average voltage

        pub total_duration: chrono::Duration, // total duration (in sec) of the interval for the current statistics
    }

    #[derive(Debug, Copy, Clone)]
    pub struct PowerBlackout {
        pub timestamp: chrono::DateTime<Local>, // start of blackout
        pub duration: chrono::Duration,         // duration
    }

    #[derive(Debug)]
    pub struct DailyPowerInfo {
        pub date: Date<Local>,
        pub stats: PowerStats,
    }

    #[derive(Debug)]
    pub struct OverallPowerInfo {
        pub start: DateTime<Local>,
        pub end: DateTime<Local>,
        pub stats: PowerStats,
        pub avg_daily_power_consumption: Option<f64>, // kWh
    }

    #[derive(Debug)]
    pub struct BlackoutInfo {
        pub blackout_count: usize,
        pub total_blackout_duration: chrono::Duration,
        pub blackouts: Vec<PowerBlackout>,
    }

    impl<'a> VoltcraftStatistics<'a> {
        pub fn new(power_data: &mut Vec<PowerEvent>) -> VoltcraftStatistics {
            VoltcraftStatistics { power_data }
        }

        pub fn daily_stats(&self) -> Vec<DailyPowerInfo> {
            // First we need the individual days in the interval
            let days = self.distinct_days();
            return days
                .into_iter()
                .map(|d| return (d, self.filter_power_data(&d))) // Filter the power items corresponding to the current date
                .map(|(d, e)| return (d, VoltcraftStatistics::compute_stats(&e))) // Compute statistics on the filtered power items
                .map(|(d, r)| DailyPowerInfo { date: d, stats: r }) // And finally build a structure to hold both the date and computed statistics
                .collect::<Vec<_>>();
        }

        pub fn overall_stats(&self) -> OverallPowerInfo {
            let mut avg_daily_power_consumption = Option::None;
            let power_stats = VoltcraftStatistics::compute_stats(&self.power_data);

            // Compute the start and end of the power data
            let start = self.power_data.first().unwrap().timestamp;
            let end = self.power_data.last().unwrap().timestamp;
            // Determine the average daily consumption
            let total_duration = end - start;
            if total_duration >= Duration::days(1) {
                // If we have more than one day worth of power data, we can do some additional power statistics
                avg_daily_power_consumption = Some(
                    power_stats.total_active_power
                        / (total_duration.num_seconds() as f64 / 86400.0),
                );
            }
            OverallPowerInfo {
                start,
                end,
                stats: power_stats,
                avg_daily_power_consumption,
            }
        }

        pub fn blackout_stats(&self) -> BlackoutInfo {
            let blackouts = &VoltcraftStatistics::compute_blackouts(&self.power_data);
            let blackout_count = blackouts.len();
            let total_blackout_duration = blackouts
                .into_iter()
                .fold(Duration::zero(), |sum, x| sum + x.duration);
            BlackoutInfo {
                blackout_count,
                total_blackout_duration,
                blackouts: blackouts.to_vec(),
            }
        }

        fn distinct_days(&self) -> Vec<Date<Local>> {
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

        fn filter_power_data(&self, day: &Date<Local>) -> Vec<PowerEvent> {
            let filtered_data = self
                .power_data
                .iter()
                .filter(|d| *day == d.timestamp.date())
                .map(|x| *x)
                .collect::<Vec<_>>();
            filtered_data
        }

        // Compute power stats on the given power events
        fn compute_stats(power_items: &Vec<PowerEvent>) -> PowerStats {
            // Total active power (in kWh) = (sum of instantaneous powers) / 60
            let power_sum = power_items.into_iter().fold(0f64, |sum, x| sum + x.power);
            let total_active_power = power_sum / 60f64; // Total active power consumption (kWh)
            let avg_active_power = power_sum / power_items.len() as f64; // Average power (kW)
            let max_active_power = power_items
                .into_iter()
                .max_by(|a, b| a.power.partial_cmp(&b.power).unwrap())
                .unwrap(); // Maximum active power (kW)

            // Total apparent power (in kVAh) = (sum of instantaneous apparent powers) / 60
            let apparent_power_sum = power_items
                .into_iter()
                .fold(0f64, |sum, x| sum + x.apparent_power);
            let total_apparent_power = apparent_power_sum / 60f64; // Total apparent power consumption (kVAh)
            let avg_apparent_power = apparent_power_sum / power_items.len() as f64; // Average power (kVA)
            let max_apparent_power = power_items
                .into_iter()
                .max_by(|a, b| a.apparent_power.partial_cmp(&b.apparent_power).unwrap())
                .unwrap(); // Maximum apparent power (kVA)

            let min_voltage = power_items
                .into_iter()
                .min_by(|a, b| a.voltage.partial_cmp(&b.voltage).unwrap())
                .unwrap(); // Minimum voltage (V)
            let max_voltage = power_items
                .into_iter()
                .max_by(|a, b| a.voltage.partial_cmp(&b.voltage).unwrap())
                .unwrap(); // Maximum voltage (V)
            let avg_voltage = &power_items.into_iter().fold(0f64, |sum, x| sum + x.voltage)
                / power_items.len() as f64; // Average voltage (V)

            let start = power_items
                .into_iter()
                .min_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap())
                .unwrap()
                .timestamp; // Start timestamp
            let end = power_items
                .into_iter()
                .max_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap())
                .unwrap()
                .timestamp; // End timestamp
            PowerStats {
                total_active_power,
                avg_active_power,
                max_active_power: *max_active_power,
                total_apparent_power,
                avg_apparent_power,
                max_apparent_power: *max_apparent_power,
                min_voltage: *min_voltage,
                max_voltage: *max_voltage,
                avg_voltage,
                total_duration: (end - start) + Duration::minutes(1),
            }
        }

        // Compute blackout stats on the given power events
        fn compute_blackouts(power_items: &Vec<PowerEvent>) -> Vec<PowerBlackout> {
            let mut blackouts = Vec::new();
            for (pe1, pe2) in power_items.iter().tuple_windows() {
                // If the gap between two subsequent timestamps is more than a minute, we've detected a blackout
                if pe2.timestamp - pe1.timestamp > Duration::minutes(1) {
                    blackouts.push(PowerBlackout {
                        timestamp: pe1.timestamp + Duration::minutes(1),
                        duration: (pe2.timestamp - pe1.timestamp) - Duration::minutes(1),
                    })
                }
            }
            blackouts
        }
    }
}
