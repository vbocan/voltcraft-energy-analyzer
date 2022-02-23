mod export;
mod voltcraft;

use colored::*;
use glob::glob;
use voltcraft::data::{PowerEvent, VoltcraftData};
use voltcraft::stats::VoltcraftStatistics;

use export::{save_parameter_history_csv, save_parameter_history_txt, save_statistics};

const PARAMETER_HISTORY_FILE_TEXT: &str = "data/parameter_history.txt";
const PARAMETER_HISTORY_FILE_CSV: &str = "data/parameter_history.csv";
const STATS_FILE_TEXT: &str = "data/stats.txt";

fn main() {
    println!(
        "{} - {}\n{} | {}",
        "Analyzer for Voltcraft Energy Logger 4000"
            .bright_white()
            .bold(),
        "v1.0".bright_yellow().bold(),
        "Valer BOCAN, PhD, CSSLP".green(),
        "https://github.com/vbocan/voltcraft-energy-decoder".blue()
    );
    let mut power_events = Vec::<PowerEvent>::new();

    for e in glob("data/*").unwrap().filter_map(Result::ok) {
        let file = e.display().to_string();
        print!("Processing file: {}...", file);
        // Open the file
        if let Ok(vdf) = VoltcraftData::from_file(&file) {
            // Parse data
            if let Ok(mut pev) = vdf.parse() {
                power_events.append(&mut pev);
                println!(" {}", "Ok".green());
            } else {
                println!(" {}", "Invalid".red());
            }
        } else {
            println!(" {}", "Failed to open".red());
        }
    }

    if !power_events.is_empty() {
        print!("Sorting power data...");
        // Chronologically sort power items (we need this to spot power blackouts)
        power_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        println!(" {}", "Done".green());
        print!("Removing duplicates from power data...");
        // Remove duplicate events based on timestamp
        power_events.dedup_by(|a, b| a.timestamp == b.timestamp);
        println!(" {}", "Done".green());

        // Write power events to text file
        print!(
            "Saving parameter history to text file {}...",
            PARAMETER_HISTORY_FILE_TEXT.bright_white()
        );
        if save_parameter_history_txt(PARAMETER_HISTORY_FILE_TEXT, &power_events).is_ok() {
            println!(" {}", "Ok".green());
        } else {
            println!(" {}", "Failed".red());
        }
        // Write power events to CSV file
        print!(
            "Saving parameter history to CSV file {}...",
            PARAMETER_HISTORY_FILE_CSV.bright_white()
        );
        if save_parameter_history_csv(PARAMETER_HISTORY_FILE_CSV, &power_events).is_ok() {
            println!(" {}", "Ok".green());
        } else {
            println!(" {}", "Failed".red());
        }
        // Compute statistics
        let stats = VoltcraftStatistics::new(&mut power_events);
        print!(
            "Saving statistics to file {}...",
            STATS_FILE_TEXT.bright_white()
        );
        if save_statistics(
            STATS_FILE_TEXT,
            &stats.overall_stats(),
            &stats.daily_stats(),
            &stats.blackout_stats(),
        )
        .is_ok()
        {
            println!(" {}", "Ok".green());
        } else {
            println!(" {}", "Failed".red());
        }
    } else {
        println!("{}", "No valid Voltcraft data files found.".yellow());
    }
    println!("{}", "Finished.".green());
}
