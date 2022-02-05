use crate::voltcraft::data::PowerEvent;
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

    pub total_apparent_power: f64,      // total apparent power (kWh)
    pub avg_apparent_power: f64,        // average apparent power (kW)
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
                power_stats.total_active_power / (total_duration.num_seconds() as f64 / 86400.0),
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
