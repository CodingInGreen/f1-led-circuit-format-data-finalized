use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct DriverData {
    pub driver_number: u32,
    pub led_num: u32,
}

#[derive(Debug)]
pub struct UpdateFrame {
    pub drivers: [Option<DriverData>; 20],
}

#[derive(Debug)]
pub struct VisualizationData {
    pub update_rate_ms: u32,
    pub frames: [UpdateFrame; 5003],
}

#[derive(Debug)]
pub struct DriverDataWithTimestamp {
    pub timestamp: String,
    pub driver_data: DriverData,
}

fn main() {
    let file = File::open("processed_race_data_all_remove_front_0s_end_stack_overflow_100k.csv").expect("Cannot open file");
    let reader = BufReader::new(file);

    let mut driver_data_vec: Vec<DriverDataWithTimestamp> = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue; // skip header
        }
        let line = line.expect("Cannot read line");
        let fields: Vec<&str> = line.split(',').collect();
        let timestamp: String = fields[2].trim().to_string();  // Adjusted to match date column
        let led_num: u32 = match fields[4].trim().parse() {    // Adjusted to match led_num column
            Ok(num) => num,
            Err(_) => {
                eprintln!("Cannot parse led_num: {:?}", fields[4].trim());
                continue;
            }
        };
        let driver_number: u32 = match fields[3].trim().parse() { // Adjusted to match driver_number column
            Ok(num) => num,
            Err(_) => {
                eprintln!("Cannot parse driver_number: {:?}", fields[3].trim());
                continue;
            }
        };

        driver_data_vec.push(DriverDataWithTimestamp {
            timestamp,
            driver_data: DriverData {
                driver_number,
                led_num,
            },
        });
    }

    // Group data by timestamp
    let mut timestamp_map: HashMap<String, Vec<DriverData>> = HashMap::new();
    for entry in driver_data_vec {
        let entry_list = timestamp_map.entry(entry.timestamp).or_insert(Vec::new());
        if !entry_list.iter().any(|d| d.driver_number == entry.driver_data.driver_number) {
            entry_list.push(entry.driver_data);
        }
    }

    let mut frames: Vec<UpdateFrame> = Vec::new();
    let total_frames = 5003; // Adjusted for 10 frames for demo purposes

    // Sort timestamps to maintain sequential order
    let mut sorted_timestamps: Vec<String> = timestamp_map.keys().cloned().collect();
    sorted_timestamps.sort();

    for timestamp in sorted_timestamps {
        let driver_data_list = &timestamp_map[&timestamp];
        let mut drivers: [Option<DriverData>; 20] = Default::default();
        for (i, driver_data) in driver_data_list.iter().take(20).enumerate() {
            drivers[i] = Some(*driver_data);
        }
        frames.push(UpdateFrame { drivers });

        // If we reach the total number of frames needed, stop processing
        if frames.len() == total_frames {
            break;
        }
    }

    // Fill remaining frames with empty UpdateFrames if necessary
    while frames.len() < total_frames {
        frames.push(UpdateFrame { drivers: [None; 20] });
    }

    let visualization_data = VisualizationData {
        update_rate_ms: 500, // Adjusted to 500ms
        frames: frames.try_into().expect("Wrong number of frames"),
    };

    let mut output_file = File::create("output.txt").expect("Cannot create output file");
    write!(
        output_file,
        "pub const VISUALIZATION_DATA: VisualizationData = {:?};",
        visualization_data
    )
    .expect("Cannot write to output file");
}
