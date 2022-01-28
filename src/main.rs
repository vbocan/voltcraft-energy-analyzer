use voltcraft_energy_decoder::{PowerEvent, VoltcraftData, VoltcraftStatistics};
extern crate glob;
use glob::glob;

fn main() {
    let mut power_items = Vec::<PowerEvent>::new();

    for e in glob("data/*").unwrap().filter_map(Result::ok) {
        let file = e.display().to_string();
        println!("Processing: {}", file);
        let vd = VoltcraftData::from_file(&file).unwrap();
        let mut pis = vd.parse().unwrap();

        power_items.append(&mut pis);
    }

    let stats = VoltcraftStatistics::new(&power_items);
    let os = stats.overall_stats();
    for pi in power_items.iter().take(5){
        println!("{:?}", pi);
    }

    println!("    OVERALL");

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
