use crate::voltcraft::data::PowerEvent;
use crate::voltcraft::stats::{BlackoutInfo, DailyPowerInfo, OverallPowerInfo};
use csv;
use std::fs::File;
use std::io::{self, Write};

pub fn save_parameter_history_txt(
    filename: &str,
    power_events: &Vec<PowerEvent>,
) -> Result<(), io::Error> {
    let mut f = File::create(filename)?;
    writeln!(f, "== PARAMETER HISTORY ==");
    writeln!(f);
    for pe in power_events {
        writeln!(
            f,
            "{} U={:.1}V I={:.3}A cosPHI={:.2} P={:.3}kW S={:.3}kVA",
            pe.timestamp.format("[%Y-%m-%d %H:%M]"),
            pe.voltage,
            pe.current,
            pe.power_factor,
            pe.power,
            pe.apparent_power
        );
    }
    Ok(())
}

pub fn save_parameter_history_csv(
    filename: &str,
    power_events: &Vec<PowerEvent>,
) -> Result<(), io::Error> {
    let mut wtr = csv::Writer::from_path(filename)?;
    wtr.write_record(&[
        "Timestamp",
        "Voltage (V)",
        "Current (A)",
        "cosPHI",
        "Active Power (kW)",
        "Apparent Power (kVA)",
    ])?;
    for pe in power_events {
        wtr.write_record(&[
            pe.timestamp.format("%Y-%m-%d %H:%M").to_string(),
            pe.voltage.to_string(),
            pe.current.to_string(),
            pe.power_factor.to_string(),
            pe.power.to_string(),
            pe.apparent_power.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn save_statistics(
    filename: &str,
    overall_stats: &OverallPowerInfo,
    daily_stats: &Vec<DailyPowerInfo>,
    blackout_stats: &BlackoutInfo,
) -> Result<(), io::Error> {
    let mut f = File::create(filename)?;
    // Statistics for the entire period
    writeln!(f, "==== OVERALL STATISTICS ==================");
    writeln!(
        f,
        "Interval: {}-{} ({})",
        overall_stats.start.format("[%Y-%m-%d %H:%M]"),
        overall_stats.end.format("[%Y-%m-%d %H:%M]"),
        format_duration(overall_stats.end - overall_stats.start)
    );
    match overall_stats.avg_daily_power_consumption {
        None => {}
        Some(d) => {
            writeln!(
                f,
                "Average consumption: {:.2}kWh/day | Projected: {:.2}kWh/month or {:.2}kWh/year.",
                d,
                d * 30.0,
                d * 365.0
            );
        }
    }
    writeln!(f);
    writeln!(f, "- ACTIVE POWER");
    writeln!(
        f,
        "Total energy consumption: {:.2}kWh.",
        overall_stats.stats.total_active_power
    );
    writeln!(
        f,
        "Peak power was {:.2}kW and occured on {}.",
        overall_stats.stats.max_active_power.power,
        overall_stats
            .stats
            .max_active_power
            .timestamp
            .format("[%Y-%m-%d %H:%M]")
    );
    writeln!(
        f,
        "Minute by minute average power: {:.2}kW.",
        overall_stats.stats.avg_active_power
    );
    writeln!(f);
    writeln!(f, "- APPARENT POWER");
    writeln!(
        f,
        "Total energy consumption: {:.2}kVAh.",
        overall_stats.stats.total_apparent_power
    );
    writeln!(
        f,
        "Peak power was {:.2}kVA and occured on {}.",
        overall_stats.stats.max_apparent_power.power,
        overall_stats
            .stats
            .max_apparent_power
            .timestamp
            .format("[%Y-%m-%d %H:%M]")
    );
    writeln!(
        f,
        "Minute by minute average power: {:.2}kVA.",
        overall_stats.stats.avg_apparent_power
    );
    writeln!(f);
    writeln!(f, "- VOLTAGE");
    writeln!(
        f,
        "Minimum voltage was {:.1}V and occured on {}.",
        overall_stats.stats.min_voltage.voltage,
        overall_stats
            .stats
            .min_voltage
            .timestamp
            .format("[%Y-%m-%d %H:%M]")
    );
    writeln!(
        f,
        "Maximum voltage was {:.1}V and occured on {}.",
        overall_stats.stats.max_voltage.voltage,
        overall_stats
            .stats
            .max_voltage
            .timestamp
            .format("[%Y-%m-%d %H:%M]")
    );
    writeln!(
        f,
        "Minute by minute average voltage: {:.1}V.",
        overall_stats.stats.avg_voltage
    );
    writeln!(f);
    writeln!(f);

    writeln!(f, "==== DAILY STATISTICS ====================");
    // Daily statistics
    for interval in daily_stats {
        writeln!(
            f,
            "{} - {} recorded activity ({:.1}%)",
            interval.date.format("[%Y-%m-%d]"),
            format_duration(interval.stats.total_duration),
            interval.stats.total_duration.num_seconds() as f64 * 100.0 / 86400.0
        );
        writeln!(
            f,
            "      Total active power: {:.2}kWh  | Average: {:.2}kW  | Maximum: {:.2}kW on {}",
            interval.stats.total_active_power,
            interval.stats.avg_active_power,
            interval.stats.max_active_power.power,
            interval
                .stats
                .max_active_power
                .timestamp
                .format("[%Y-%m-%d %H:%M]")
        );
        writeln!(
            f,
            "    Total apparent power: {:.2}kVAh | Average: {:.2}kVA | Maximum: {:.2}kVA on {}",
            interval.stats.total_active_power,
            interval.stats.avg_active_power,
            interval.stats.max_active_power.power,
            interval
                .stats
                .max_active_power
                .timestamp
                .format("[%Y-%m-%d %H:%M]")
        );
        writeln!(
            f,
            "    Voltage: Average: {:.1}V | Minimum: {:.1}V on {} | Maximum: {:.1}V on {}",
            interval.stats.avg_voltage,
            interval.stats.min_voltage.voltage,
            interval
                .stats
                .min_voltage
                .timestamp
                .format("[%Y-%m-%d %H:%M]"),
            interval.stats.max_voltage.voltage,
            interval
                .stats
                .max_voltage
                .timestamp
                .format("[%Y-%m-%d %H:%M]")
        );
        writeln!(f);
    }

    writeln!(f);
    // Blackout history
    writeln!(f, "==== BLACKOUT HISTORY ====================");
    writeln!(
        f,
        "{} blackout(s) for a total of {}.",
        blackout_stats.blackout_count,
        format_duration(blackout_stats.total_blackout_duration)
    );
    writeln!(f);
    for be in &blackout_stats.blackouts {
        writeln!(
            f,
            "{} Duration: {}",
            be.timestamp.format("[%Y-%m-%d %H:%M]"),
            format_duration(be.duration),
        );
    }
    Ok(())
}

fn format_duration(duration: chrono::Duration) -> String {
    let minutes = (duration.num_seconds() / 60) % 60;
    let hours = (duration.num_seconds() / 3600) % 24;
    let days = duration.num_seconds() / 86400;
    if days > 0 {
        format!("{:0>2}d:{:0>2}h:{:0>2}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{:0>2}h:{:0>2}m", hours, minutes)
    } else {
        format!("{:0>2}m", minutes)
    }
}
