#![deny(missing_docs)]

//! A basic post-processor for adding Prusaslicer gcode attributes to the beginning of 
//! gcode files to help the Ankermake M5 printer to correctly estimate print times and
//! material usage.

use std::env::{Args, args};
use std::fs::File;
use std::io::{BufReader, BufRead, Lines, BufWriter, Write};
use std::path::Path;

/// Prusaslicer attribute for the estimated printing time. Formatted as "XXh YYm ZZs" string
pub const PRUSA_ESTIMATED_PRINTING_TIME: &str = "estimated printing time";
/// Prusaslicer attribute for the estimated material usage. Formatted in millimeters, to 2 decimal places
pub const PRUSA_FILAMENT_USED_MM: &str = "filament used [mm]";

/// Ankermake attribute for the estimated printing time. Formatted as integer number of seconds.
pub const ANKERMAKE_PRINTING_TIME: &str = "TIME";
/// Ankermake attribute for the estimated material usage. Formatted in meters to 5 decimal places.
pub const ANKERMAKE_FILAMENT_USED_M: &str = "Filament used";
/// The gcode flavour, always Marlin
pub const ANKERMAKE_FLAVOUR: &str = "FLAVOR";

/// Potential errors that can be encountered while parsing the gcode
#[derive(Debug)]
pub enum ParsingError {
    /// While attempting to extract a value from a line, no value was found
    MissingValue(String),
    /// An attempt to parse a string into the specified type failed
    StringParsingError(&'static str, String)
}

/// Selection of fields that we're interested in reformatting for the Ankermake M5 to understand.
pub enum InterestingFields {
    /// Time taken to print, represented as seconds
    Time(u64),
    /// Amount of filament used during printing, in um x10(0.01 mm)
    FilamentUsed(u64),
    /// gcode flavour. always Marlin
    Flavour(String)
}

impl ToString for InterestingFields {
    fn to_string(&self) -> String {
        use InterestingFields::*;
        match self {
            Time(seconds) => format!(";{}:{}", ANKERMAKE_PRINTING_TIME, seconds),
            FilamentUsed(length_umx10) => format!(";{}: {}m", ANKERMAKE_FILAMENT_USED_M, (*length_umx10 as f64) / 100000.0),
            Flavour(flavour) => format!(";{}:{}", ANKERMAKE_FLAVOUR, flavour)
        }
    }
}

/// Given a line, attempt to parse the value into an integer number of seconds
pub fn extract_time_data_as_seconds(attribute: &str) -> Result<u64, ParsingError> {
    // After splitting on the equals sign, skipping the left hand side and trimming the resulting string
    // we should just have "XXh YYm ZZs"
    let value = if let Some(string_value) = attribute.split('=').skip(1).next() {
        string_value.trim()
    } else {
        return Err(ParsingError::MissingValue(attribute.to_string()));
    };

    let time: u64 = value.split(' ').into_iter().map(|value| {
        if value.ends_with('h') {
            value.strip_suffix("h").unwrap().parse::<u64>().unwrap() * 60 * 60
        } else if value.ends_with('m') {
            value.strip_suffix("m").unwrap().parse::<u64>().unwrap() * 60
        } else if value.ends_with('s') {
            value.strip_suffix("s").unwrap().parse::<u64>().unwrap()
        } else {
            panic!()
        }
    }).sum();

    Ok(time)

}

/// Given a line, attempt to extract how many 10s of micrometers of filament are predicted to be used.
pub fn extract_filament_used_as_um_x10(attribute: &str) -> Result<u64, ParsingError> {
    // After splitting on the equals sign, skipping the left hand side and trimming the resulting string
    // we should just have "XXXX.YY", our length in millimeters.
    let value = if let Some(string_value) = attribute.split('=').skip(1).next() {
        string_value.trim()
    } else {
        return Err(ParsingError::MissingValue(attribute.to_string()));
    };

    // Split on the decimal place, then just collect back into a string, which we should be able to parse
    // into an integer value.
    let integer_value_str: String = value.split('.').collect();

    if let Ok(parsed_integer) = integer_value_str.parse() {
        Ok(parsed_integer)
    } else {
        Err(ParsingError::StringParsingError("u64", integer_value_str))
    }
}

/// Process the lines in the file, pulling out the attributes that we're interested in and reinserting them in the header for the
/// file. Returns the new file contents that should be written to the disk.
pub fn process_lines(lines: Lines<impl BufRead>) -> String {
    let mut interesting_fields: Vec<InterestingFields> = vec![InterestingFields::Flavour("Marlin".into())];

    // Ditch erroneous lines
    let lines: Vec<String> = lines.into_iter().flatten().collect();
    
    lines.iter().for_each(|line|{
        // Check that our line has enough data on it to have _something_ after skipping the first
        // 2 bytes
        if line.len() > 3 {
            let trimmed_line = &line[2..];
            if trimmed_line.starts_with(PRUSA_ESTIMATED_PRINTING_TIME) {
                interesting_fields.push(InterestingFields::Time(extract_time_data_as_seconds(trimmed_line).unwrap()))
            } else if trimmed_line.starts_with(PRUSA_FILAMENT_USED_MM) {
                interesting_fields.push(InterestingFields::FilamentUsed(extract_filament_used_as_um_x10(trimmed_line).unwrap()))
            }
        }
    });

    let mut file_contents: Vec<String> = interesting_fields.into_iter().map(|val| {
        val.to_string()
    }).collect();

    file_contents.extend(lines.into_iter());

    file_contents.join("\n")
}

/// Attempt to open the file at the location described in the string, displaying the OS error if the file couldn't be opened for
/// some reason.
pub fn process_file(file_path_string: String) {
    let file_path: &Path = Path::new(&file_path_string);
    
    let file: File = match File::open(file_path) {
        Ok(file) => file,
        Err(file_opening_error) => {
            eprintln!("Failed to open file at \"{file_path_string}\": {file_opening_error:?}");
            return;
        }
    };

    let file_reader: BufReader<File> = BufReader::new(file);

    let new_file_contents: String = process_lines(file_reader.lines());

    let file: File = match File::create(file_path) {
        Ok(file) => file,
        Err(file_opening_error) => {
            eprintln!("Failed to open file at \"{file_path_string}\": {file_opening_error:?}");
            return;
        }
    };

    let mut file_writer: BufWriter<File> = BufWriter::new(file);
    file_writer.write_all(new_file_contents.as_bytes()).unwrap();

}

fn main() {
    let arguments: Args = args();

    // Skip first argument, as that's this program
    arguments.skip(1).for_each(|file_path_string|{
        process_file(file_path_string)
    })
}
