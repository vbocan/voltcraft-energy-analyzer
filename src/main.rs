mod export;
mod voltcraft;

use glob::glob;
use voltcraft::data::{PowerEvent, VoltcraftData};
use voltcraft::stats::VoltcraftStatistics;

use export::{save_parameter_history_csv, save_parameter_history_txt, save_statistics};

const PARAMETER_HISTORY_FILE_TEXT: &'static str = "data/parameter_history.txt";
const PARAMETER_HISTORY_FILE_CSV: &'static str = "data/parameter_history.csv";
const STATS_FILE_TEXT: &'static str = "data/stats.txt";

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

    if power_events.len() > 0 {
        println!("Sorting power data...");
        // Chronologically sort power items (we need this to spot power blackouts)
        power_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        // Write power events to text file
        print!(
            "Saving parameter history to text file {}...",
            PARAMETER_HISTORY_FILE_TEXT
        );
        if let Ok(_) = save_parameter_history_txt(PARAMETER_HISTORY_FILE_TEXT, &power_events) {
            println!(" OK");
        } else {
            println!(" Failed!");
        }
        // Write power events to CSV file
        print!(
            "Saving parameter history to CSV file {}...",
            PARAMETER_HISTORY_FILE_CSV
        );
        if let Ok(_) = save_parameter_history_csv(PARAMETER_HISTORY_FILE_CSV, &power_events) {
            println!(" OK");
        } else {
            println!(" Failed!");
        }
        // Compute statistics
        let stats = VoltcraftStatistics::new(&mut power_events);
        print!("Saving statistics to file {}...", STATS_FILE_TEXT);
        if let Ok(_) = save_statistics(
            STATS_FILE_TEXT,
            &stats.overall_stats(),
            &stats.daily_stats(),
            &stats.blackout_stats(),
        ) {
            println!(" OK");
        } else {
            println!(" Failed!");
        }
    } else {
        println!("No Voltcraft data found.");
    }
    println!("Finished.");
}
