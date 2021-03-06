mod export;
mod voltcraft;

use colored::*;
use glob::glob;
use std::env;
use std::fs;
use std::time::Instant;
use voltcraft::data::{PowerEvent, VoltcraftData};
use voltcraft::stats::VoltcraftStatistics;

use export::{save_parameter_history_csv, save_parameter_history_txt, save_statistics};

const PARAMETER_HISTORY_FILE_TEXT: &str = "voltcraft_history.txt";
const PARAMETER_HISTORY_FILE_CSV: &str = "voltcraft_history.csv";
const STATS_FILE_TEXT: &str = "voltcraft_stats.txt";

fn main() {
    // Print welcome text
    display_welcome();
    // Process command-line arguments
    let args: Vec<String> = env::args().collect();

    let (mut input_dir, mut output_dir) = {
        if args.len() == 3 {
            // We have both the input and the output folder
            (String::from(&args[1]), String::from(&args[2]))
        } else if args.len() == 2 {
            // We only have one argument, check whether help is requested
            if args[1].eq_ignore_ascii_case("-h")
                || args[1].eq_ignore_ascii_case("--help")
                || args[1].eq_ignore_ascii_case("/?")
            {
                display_help();
                return;
            }
            (String::from(&args[1]), String::from("./"))
        } else {
            // No folder given
            (String::from("./"), String::from("./"))
        }
    };

    // Create output folder
    if fs::create_dir_all(&output_dir).is_err() {
        println!(
            "{} {}",
            "Failed to create folder".red(),
            output_dir.bright_red()
        );
        return;
    }

    // Add a trailing / to folders (if doesn't exist already)
    if !input_dir.ends_with('/') {
        input_dir.push('/');
    }
    if !output_dir.ends_with('/') {
        output_dir.push('/');
    }

    println!(
        "Reading data files from folder '{}'.",
        input_dir.bright_white()
    );
    println!(
        "Writing statistics to folder '{}'.",
        output_dir.bright_white()
    );

    let start_time = Instant::now();
    // Initialize the vector that stores incoming power events
    let mut power_events = Vec::<PowerEvent>::new();

    // Parse input folder
    input_dir.push('*');

    // Read the input directory and process each file
    let mut file_count = 0;
    for e in glob(input_dir.as_str()).unwrap().filter_map(Result::ok) {
        let file = e.display().to_string();
        print!("Processing file: {}...", file);
        // Open the file
        if let Ok(vdf) = VoltcraftData::from_file(&file) {
            // Parse data
            if let Ok(mut pev) = vdf.parse() {
                power_events.append(&mut pev);
                file_count += 1;
                println!(" {}", "Ok".green());
            } else {
                println!(" {}", "Invalid".red());
            }
        } else {
            println!(" {}", "Failed to open".red());
        }
    }

    // Process power events accrued from the parsed data files
    if !power_events.is_empty() {
        // Chronologically sort power items (we need this to spot power blackouts)
        print!("Sorting power data...");
        power_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        println!(" {}", "Done".green());
        // Remove duplicate events based on timestamp
        print!("Removing duplicates from power data...");
        power_events.dedup_by(|a, b| a.timestamp == b.timestamp);
        println!(" {}", "Done".green());
        // Write power events to text file
        let mut target_path = output_dir.clone();
        target_path.push_str(PARAMETER_HISTORY_FILE_TEXT);
        print!(
            "Saving parameter history to text file {}...",
            PARAMETER_HISTORY_FILE_TEXT.bright_white()
        );
        if save_parameter_history_txt(target_path.as_str(), &power_events).is_ok() {
            println!(" {}", "Ok".green());
        } else {
            println!(" {}", "Failed".red());
        }
        // Write power events to CSV file
        let mut target_path = output_dir.clone();
        target_path.push_str(PARAMETER_HISTORY_FILE_CSV);
        print!(
            "Saving parameter history to CSV file {}...",
            PARAMETER_HISTORY_FILE_CSV.bright_white()
        );
        if save_parameter_history_csv(target_path.as_str(), &power_events).is_ok() {
            println!(" {}", "Ok".green());
        } else {
            println!(" {}", "Failed".red());
        }
        // Compute statistics
        let mut target_path = output_dir.clone();
        target_path.push_str(STATS_FILE_TEXT);
        let stats = VoltcraftStatistics::new(&mut power_events);
        print!(
            "Saving statistics to file {}...",
            STATS_FILE_TEXT.bright_white()
        );
        if save_statistics(
            target_path.as_str(),
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

    let duration = start_time.elapsed();

    if file_count > 0 {
        println!("Processed {} files in {:?}.", file_count, duration);
    }
    println!("{}", "Finished.".green());
}

fn display_welcome() {
    println!(
        "{} - {} {}\n{} | {}",
        "Analyzer for Voltcraft Energy Logger 4000"
            .bright_white()
            .bold(),
        "v1.0".bright_yellow().bold(),
        "(My first foray into the Rust programming language)".italic(),
        "Valer BOCAN, PhD, CSSLP".green(),
        "https://github.com/vbocan/voltcraft-energy-analyzer".blue()
    );
    println!(
        "Type {} | {} | {} to get help.\n",
        "/?".yellow(),
        "-h".yellow(),
        "--help".yellow()
    );
}

fn display_help() {
    println!("{} <input folder> <output folder>\n\t- Decode Voltcraft files from a folder and output statistics in different folder.",
        "voltcraft_energy_analyzer".bright_white());
    println!("{} <input folder>\n\t- Decode Voltcraft files from a folder and output statistics in the current folder.",
        "voltcraft_energy_analyzer".bright_white());
    println!(
        "{}\n\t- Decode Voltcraft files from and place the statistics in the current folder.\n",
        "voltcraft_energy_analyzer".bright_white()
    );
}
