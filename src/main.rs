use std::fs::OpenOptions;
use std::io::Write;
use voltcraft_energy_decoder::{PowerEvent, VoltcraftData, VoltcraftStatistics};
extern crate glob;
use glob::glob;

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
    // Write power events to file
    write_to_file(&power_events);
    // Write blackouts
    let stats = VoltcraftStatistics::new(&mut power_events);
    println!("    BLACKOUTS");
    let blackouts = stats.blackout_stats();
    for bo in blackouts {
        println!("{:?}", bo);
    }
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

fn write_to_file(power_events: &Vec<PowerEvent>) {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true) // Optionally create the file if it doesn't already exist
        .open("data/output.txt")
        .expect("Unable to open/create file");
    for pe in power_events {
        let fs = format!(
            "[{}] U={:.1}V I={:.3}A cosPHI={:.2} P={:.3} S={:.3}\n",
            pe.timestamp, pe.voltage, pe.current, pe.power_factor, pe.power, pe.apparent_power
        );
        f.write_all(fs.as_bytes()).expect("Unable to write data");
    }
}
