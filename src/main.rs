use std::fs::OpenOptions;
use std::io::Write;
use voltcraft_energy_decoder::{PowerBlackout, PowerEvent, VoltcraftData, VoltcraftStatistics};
extern crate glob;
use glob::glob;

const PARAMETER_HISTORY_FILE_TEXT: &'static str = "data/parameter_history.txt";
//const PARAMETER_HISTORY_FILE_CSV: &'static str = "data/parameter_history.csv";
const BLACKOUT_HISTORY_FILE_TEXT: &'static str = "data/blackout_history.txt";

fn main() {
    let mut power_events = Vec::<PowerEvent>::new();

    for e in glob("data/*").unwrap().filter_map(Result::ok) {
        let file = e.display().to_string();
        println!("Processing file: {}...", file);
        // Open the file
        if let Ok(vdf) = VoltcraftData::from_file(&file) {
            // Parse data
            if let Ok(mut pev) = vdf.parse() {
                power_events.append(&mut pev);
            } else {
                eprintln!("Invalid data, probably not a Voltcraft file.");
            }
        } else {
            eprintln!("Failed to open file.");
        }
    }

    // Chronologically sort power items (we need this to spot power blackouts)
    power_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    // Write power events to text file
    save_parameter_history(PARAMETER_HISTORY_FILE_TEXT, &power_events);
    // Compute statistics
    let stats = VoltcraftStatistics::new(&mut power_events);
    let blackouts = stats.blackout_stats();
    save_blackout_history(BLACKOUT_HISTORY_FILE_TEXT, &blackouts);

    // Write overall statistics
    println!("    OVERALL");
    let os = stats.overall_stats();
    println!("ACTIVE POWER ---------------------------------------------");
    println!(
        "Total active energy consumption: {:.2} kWh.",
        os.total_active_power
    );
    println!(
        "Maximum power was {:.2} kW and occured on {}.",
        os.max_active_power.power, os.max_active_power.timestamp
    );
    println!(
        "Average power during the day: {:.2} kW.",
        os.avg_active_power
    );

    println!("APPARENT POWER -------------------------------------------");
    println!(
        "Total apparent power consumed: {:.2} kVAh.",
        os.total_apparent_power
    );
    println!(
        "Maximum apparent power was {:.2} kVA and occured on {}.",
        os.max_apparent_power.power, os.max_apparent_power.timestamp
    );
    println!(
        "Average apparent power during the day: {:.2} kVA.",
        os.avg_apparent_power
    );

    println!("VOLTAGE --------------------------------------------------");
    println!(
        "Minimum voltage was {:.2}V and occured on {}.",
        os.min_voltage.voltage, os.min_voltage.timestamp
    );
    println!(
        "Maximum voltage was {:.2}V and occured on {}.",
        os.max_voltage.voltage, os.max_voltage.timestamp
    );
    println!("Average voltage during the day: {:.2}V.", os.avg_voltage);
    println!("");

    // Write daily statistics
    let ds = stats.daily_stats();
    for interval in ds {
        println!("    [Date: {:?}]", interval.date);

        println!("ACTIVE POWER ---------------------------------------------");
        println!(
            "Total active energy consumption: {:.2} kWh.",
            interval.stats.total_active_power
        );
        println!(
            "Maximum power was {:.2} kW and occured on {}.",
            interval.stats.max_active_power.power, interval.stats.max_active_power.timestamp
        );
        println!(
            "Average power during the day: {:.2} kW.",
            interval.stats.avg_active_power
        );

        println!("APPARENT POWER -------------------------------------------");
        println!(
            "Total apparent power consumed: {:.2} kVAh.",
            interval.stats.total_apparent_power
        );
        println!(
            "Maximum apparent power was {:.2} kVA and occured on {}.",
            interval.stats.max_apparent_power.power, interval.stats.max_apparent_power.timestamp
        );
        println!(
            "Average apparent power during the day: {:.2} kVA.",
            interval.stats.avg_apparent_power
        );

        println!("VOLTAGE --------------------------------------------------");
        println!(
            "Minimum voltage was {:.2}V and occured on {}.",
            interval.stats.min_voltage.voltage, interval.stats.min_voltage.timestamp
        );
        println!(
            "Maximum voltage was {:.2}V and occured on {}.",
            interval.stats.max_voltage.voltage, interval.stats.max_voltage.timestamp
        );
        println!(
            "Average voltage during the day: {:.2}V.",
            interval.stats.avg_voltage
        );
        println!("");
    }
}

fn save_parameter_history(filename: &str, power_events: &Vec<PowerEvent>) {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .expect("Unable to create file");

    f.write("== PARAMETER HISTORY ==\n\n".as_bytes())
        .expect("Unable to write data");
    for pe in power_events {
        let fs = format!(
            "{} U={:.1}V I={:.3}A cosPHI={:.2} P={:.3} S={:.3}\n",
            pe.timestamp.format("[%Y-%m-%d %H:%M]"),
            pe.voltage,
            pe.current,
            pe.power_factor,
            pe.power,
            pe.apparent_power
        );
        f.write_all(fs.as_bytes()).expect("Unable to write data");
    }
}

fn save_blackout_history(filename: &str, blackout_events: &Vec<PowerBlackout>) {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .expect("Unable to create file");

    f.write("== BLACKOUT HISTORY ==\n\n".as_bytes())
        .expect("Unable to write data");
    for be in blackout_events {
        let fs = format!(
            "{} Duration: {}\n",
            be.timestamp.format("[%Y-%m-%d %H:%M]"),
            format_duration(be.duration),
        );
        f.write_all(fs.as_bytes()).expect("Unable to write data");
    }
}

fn format_duration(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds() % 60;
    let minutes = (duration.num_seconds() / 60) % 60;
    let hours = (duration.num_seconds() / 60) / 60;
    format!("{:0>2}h:{:0>2}m:{:0>2}s", hours, minutes, seconds)
}
