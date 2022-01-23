use voltcraft_energy_decoder::{PowerItem, VoltcraftData};
extern crate glob;
use glob::glob;

fn main() {
    let mut power_items = Vec::<PowerItem>::new();

    for e in glob("data/*").unwrap().filter_map(Result::ok) {
        let file = e.display().to_string();
        println!("Processing: {}", file);
        let vd = VoltcraftData::from_file(&file).unwrap();
        let mut pis = vd.parse().unwrap();

        power_items.append(&mut pis);
    }

    println!("Found {} power items.", power_items.len());
    let mints = power_items.iter().min_by_key(|x| x.timestamp);
    let maxts = power_items.iter().max_by_key(|x| x.timestamp);
    println!("Start time {:?}", mints.unwrap().timestamp);
    println!("End time {:?}", maxts.unwrap().timestamp);
    // Sort power items chronologically
    power_items.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    for p in power_items.iter().take(5) {
        println!("{:?}", &p);
    }
}
