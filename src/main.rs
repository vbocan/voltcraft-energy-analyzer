mod export;
mod voltcraft;

use glob::glob;
use voltcraft::data::{PowerEvent, VoltcraftData};
use voltcraft::stats::VoltcraftStatistics;

use export::{save_parameter_history_csv, save_parameter_history_txt, save_statistics};

const PARAMETER_HISTORY_FILE_TEXT: &str = "data/parameter_history.txt";
const PARAMETER_HISTORY_FILE_CSV: &str = "data/parameter_history.csv";
const STATS_FILE_TEXT: &str = "data/stats.txt";

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

    if !power_events.is_empty() {
        println!("Sorting power data...");
        // Chronologically sort power items (we need this to spot power blackouts)
        power_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        println!("Removing duplicates from power data...");
        // Remove duplicate events based on timestamp
        power_events.dedup_by(|a, b| a.timestamp == b.timestamp);

        // Write power events to text file
        print!(
            "Saving parameter history to text file {}...",
            PARAMETER_HISTORY_FILE_TEXT
        );
        if save_parameter_history_txt(PARAMETER_HISTORY_FILE_TEXT, &power_events).is_ok() {
            println!(" OK");
        } else {
            println!(" Failed!");
        }
        // Write power events to CSV file
        print!(
            "Saving parameter history to CSV file {}...",
            PARAMETER_HISTORY_FILE_CSV
        );
        if save_parameter_history_csv(PARAMETER_HISTORY_FILE_CSV, &power_events).is_ok() {
            println!(" OK");
        } else {
            println!(" Failed!");
        }
        // Compute statistics
        let stats = VoltcraftStatistics::new(&mut power_events);
        print!("Saving statistics to file {}...", STATS_FILE_TEXT);
        if save_statistics(
            STATS_FILE_TEXT,
            &stats.overall_stats(),
            &stats.daily_stats(),
            &stats.blackout_stats(),
        )
        .is_ok()
        {
            println!(" OK");
        } else {
            println!(" Failed!");
        }
    } else {
        println!("No Voltcraft data found.");
    }
    println!("Finished.");
}
