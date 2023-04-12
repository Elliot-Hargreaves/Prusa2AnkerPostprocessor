/// Parameter taken by a gcode instruction
#[derive(PartialEq)]
pub struct GCodeParameter {
    identifier: u8,
    value: f32,
}

/// gcode instruction
#[derive(Eq, PartialEq)]
pub struct GCodeInstruction {
    alpha: u8,
    int: u16,
}

/// Feature types that are annotated in the gcode by PrusaSlicer
pub enum FeatureType {
    /// Section of custom gcode
    Custom,
    /// Printing of a skirt or brim
    SkirtOrBrim,
    /// Regular perimeter
    Perimeter,
    /// External perimeter
    ExternalPerimeter,
    /// Ironing section(Top layer(s) smoothing)
    Ironing,
    /// Top layer(s) infill
    TopSolidInfill,
    /// Solid interior infill
    SolidInfill,
    /// Some unrecognised feature
    Unknown(String),
}

/// A comment in the gcode, preceded by ';'
pub enum GCodeComment {
    /// Unrecognised comment, assumed to be innocuous
    Misc(String),
    /// Annotation of printing a feature of the model
    FeatureTypeAnnotation(FeatureType),
    /// Annotation of a layer change in the model
    LayerChange {
        /// The height of the new layer(not necessarily the same across all layers!)
        layer_height: f32,
        /// The absolute z_height of the layer
        z_height: f32,
    },
    /// Metadata attached to the gcode for storing the information used during slicing, for debug
    /// and to help gcode previews
    Metadata {
        /// The name of the property
        property: String,
        /// The value(s) of the property. Captured as a single string, manipulating the string should
        /// be done elsewhere
        value: String,
    },
}

/// Representation of the different lines we expect to encounter while processing the gcode.
pub enum GCodeLine {
    /// A gcode instruction, results in some action by the printer
    Instruction {
        /// The instruction the printer will follow
        instruction: GCodeInstruction,
        /// Any parameters that are used while processing the instruction
        parameters: Vec<GCodeParameter>,
    },
    /// A comment in the gcode.
    Comment(GCodeComment),
}
