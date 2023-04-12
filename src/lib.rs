#![deny(missing_docs)]
//! Library for extracting values from g-code produced by Prusaslicer for translating to a
//! format understood by the Ankermake M5 printer and slicer.

use std::error::Error;

/// Module for working with gcode
pub mod gcode;

/// A function type that attempts to convert from Prusaslicer values to Ankermake values. Accepts the
/// Prusaslicer metadata value as a string, returning either the transformed string or an error.
pub type MetadataTranslationFn = fn(metadata_value: String) -> Result<String, Box<dyn Error>>;

/// A property of the G-Code that should be translated for
/// the Ankermake M5.
pub enum MetadataProperty {
    /// A constant property of the G-Code(e.g. gcode flavour)
    Constant {
        /// Property name
        name: &'static str,
        /// Property value
        value: &'static str,
    },
    /// A field that is extracted from the Prusa gcode and translated
    /// into the appropriate format for the Ankermake M5
    Field {
        /// The field name output from PrusaSlicer
        prusa: &'static str,
        /// The field name the Ankermake M5 expects
        anker: &'static str,
        /// A function for performing the translation, if
        /// translation is required.
        translate_fn: Option<MetadataTranslationFn>,
    },
}

/// List of metadata properties that should be extracted from the Prusaslicer gcode for inserting into the gcode
/// for the Ankermake M5 to find.
pub const METADATA_PROPERTIES: &[MetadataProperty] = &[
    MetadataProperty::Constant {
        name: "FLAVOR",
        value: "Marlin",
    },
    // TODO confirm whether this impacts print speed, and whether this should be picked up from somewhere(e.g. max print speed?)
    MetadataProperty::Constant {
        name: "Print Mode",
        value: "fast",
    },
    // TODO confirm whether this is affected by AI mode
    MetadataProperty::Constant {
        name: "CompileMode",
        value: "Executable File",
    },
    MetadataProperty::Field {
        prusa: "filament_settings_id",
        anker: "Filament Name",
        translate_fn: None,
    },
    MetadataProperty::Field {
        prusa: "nozzle_diameter",
        anker: "Machine Nozzle Size",
        translate_fn: None,
    },
    MetadataProperty::Field {
        prusa: "max_print_speed",
        anker: "MAXSPEED",
        translate_fn: None,
    },
];

/// Ensure that we never end up with metadata properties that are defined multiple times since there aren't any properties that
/// should be defined more than once
#[test]
fn assert_no_duplicate_metadata_properties() {
    METADATA_PROPERTIES.iter().for_each(|property| {
        let anker_field_name = match property {
            MetadataProperty::Constant { name, value: _ } => name.clone(),
            MetadataProperty::Field {
                prusa: _,
                anker,
                translate_fn: _,
            } => anker.clone(),
        };
        assert_eq!(
            1,
            METADATA_PROPERTIES
                .iter()
                .filter(|other| match other {
                    MetadataProperty::Constant { name, value: _ } => anker_field_name == *name,
                    MetadataProperty::Field {
                        prusa: _,
                        anker,
                        translate_fn: _,
                    } => anker_field_name == *anker,
                })
                .count()
        );
    })
}
