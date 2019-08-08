//! NetLogo `.dat` files parser.
//!
//! This library can be used to parse files made by manual exporting
//! or produced by `export-world` function.
//!
//! ## Implementation details
//! NetLogo `.dat` files have header and several sections.
//!
//! To parse the file, we fully read it to memory, split to sections
//! and then parse each section using csv.

use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;

mod value;
use value::Value;

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
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct Patch {
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    #[serde(flatten)]
    custom: HashMap<String, Value>,
}

/// Parse NetLogo world from a str.
pub fn parse_str(data: &str) ->  Result<NetLogoWorld, Box<dyn Error>> {
    parse(data.as_bytes())
}

/// Parse NetLogo world from a reader.
pub fn parse(reader: impl Read) -> Result<NetLogoWorld, Box<dyn Error>> {
    let mut headers = None;
    let mut section = Section::Header;
    let mut world = NetLogoWorld::default();

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(reader);

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
                world.output.push(record.deserialize(headers.as_ref())?);
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
