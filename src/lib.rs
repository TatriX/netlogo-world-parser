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
    /// Unimplemented ;(
    pub plots: (),
}

#[derive(Debug, Deserialize, Default)]
pub struct Globals {
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
pub struct Turle {}

#[derive(Debug, Deserialize)]
pub struct Patch {}

#[derive(Debug, Deserialize)]
pub struct Link {}

pub fn parse(data: &str) -> Result<NetLogoWorld, Box<dyn Error>> {
    let mut headers = None;
    let mut section = Section::Header;
    let mut world = NetLogoWorld::default();

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());
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
            Section::Globals => {
                world.globals = record.deserialize(headers.as_ref())?;
            }
            Section::Turtles => {
                world.turtles.push(record.deserialize(headers.as_ref())?);
            }
            Section::Output => {
                world.output.push(record.deserialize(headers.as_ref())?);
            }
            _ => {}
        }
    }
    println!("WORLD: {:?}", &world);
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
    fn has_headers(&self) -> bool {
        match self {
            Section::Header | Section::Output | Section::Plots | Section::Extenstions => false,
            _ => true,
        }
    }
}
