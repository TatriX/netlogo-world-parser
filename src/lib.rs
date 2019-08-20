//! NetLogo `.dat` files parser.
//!
//! This library can be used to parse files made by manual exporting
//! or produced by `export-world` function.
//!
//! ## Implementation details
//! NetLogo `.dat` files have a header and several sections.  Every
//! section has a heading with a name of a section and optionally a
//! header csv row followed by the csv data.
//!
//! ### Parsing
//! The data is parsed as `csv` and interpreted at the same time line
//! by line. After finding a header the section is read. If a section
//! is expected to have a header, it's read first.
//!
//! ### Parsed data format
//! Data is typed and uses `custom` hashmap for added properties.
//!
//! TODO: Consider saving "raw" csv data such that a user could
//! deserialize it to his own data structure.

use serde::Deserialize;
use std::error::Error;
use std::io::Read;

mod value;
#[cfg(feature = "custom-fields")]
use value::Value;
#[cfg(feature = "custom-fields")]
use std::collections::HashMap;

/// Representation of a NetLogo World.
#[derive(Debug, Deserialize, Default)]
pub struct NetLogoWorld {
    pub random_state: Vec<i64>,
    pub globals: Globals,
    pub output: Vec<String>,
    pub turtles: Vec<Turle>,
    pub patches: Vec<Patch>,
    pub links: Vec<String>,
    /// Unimplemented ;(
    pub plots: (),
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Globals {
    pub min_pxcor: i64,
    pub max_pxcor: i64,
    pub min_pycor: i64,
    pub max_pycor: i64,
    pub ticks: usize,
    #[cfg(feature = "custom-fields")]
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

impl Globals {
    /// Get custom field if any.
    ///
    /// Can be used like this:
    /// ```
    /// u64::try_from(world.globals.get("foo").expect("no foo").to_owned())
    /// ```
    #[cfg(feature = "custom-fields")]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.custom.get(key)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Turle {
    who: usize,
    color: usize,
    xcor: i64,
    ycor: i64,
    #[cfg(feature = "custom-fields")]
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct Patch {
    #[cfg(feature = "custom-fields")]
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    #[cfg(feature = "custom-fields")]
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

/// Parse NetLogo world from a str.
pub fn parse_str(data: &str) -> Result<NetLogoWorld, Box<dyn Error>> {
    parse(data.as_bytes())
}

/// Parse NetLogo world from a reader.
pub fn parse(reader: impl Read) -> Result<NetLogoWorld, Box<dyn Error>> {
    let mut headers = None;
    let mut section = Section::Header;
    let mut world = NetLogoWorld::default();

    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(reader);

    for record in rdr.records().map(|record| record.expect("parse error")) {
        // First check if we are looking on a new section
        if let Ok(new_section) = record.deserialize::<Section>(None) {
            section = new_section;
            headers = None; // reset header
            continue;
        }

        // No header? Read one.
        if section.has_headers() && headers.is_none() {
            headers = Some(record);
            continue;
        }

        match section {
            Section::RandomState => {
                world.random_state = record.deserialize(headers.as_ref())?;
            }
            Section::Globals => {
                world.globals = record.deserialize(headers.as_ref())?;
            }
            Section::Turtles => {
                world.turtles.push(record.deserialize(headers.as_ref())?);
            }
            Section::Output => {
                world.output = parse_output(record.deserialize(headers.as_ref())?);
            }
            Section::Patches => {
                world.patches.push(record.deserialize(headers.as_ref())?);
            }
            Section::Links => {
                world.links.push(record.deserialize(headers.as_ref())?);
            }
            _ => {
                // skip the rest for now
            }
        }
    }
    Ok(world)
}

// Internal stuff

/// Known file sections
#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Section {
    Header,
    RandomState,
    Globals,
    Turtles,
    Patches,
    Links,
    Output,
    Plots,
    Extenstions,
}

impl Section {
    /// Whether we expect a header after a section name.
    fn has_headers(&self) -> bool {
        match self {
            Section::Header | Section::Output | Section::Plots | Section::Extenstions => false,
            _ => true,
        }
    }
}

// TODO: write tests
/// Parse "OUTPUT" section.
///
/// Remove surrounding double quotes and split the string on escaped
/// newlines.
fn parse_output(output: &str) -> Vec<String> {
    output
        .trim_matches('"')
        .split("\\n")
        .map(String::from)
        .collect()
}
