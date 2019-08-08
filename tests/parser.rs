use netlogo_world_parser::parse_str;
use std::convert::{TryFrom, TryInto};

#[test]
fn parse_valid_file() {
    let data = include_str!("../tests/ants.dat");
    let world = parse_str(data).expect("parse failed");
    assert_eq!(world.turtles.len(), 6);

    let population = world
        .globals
        .get("population")
        .expect("no population")
        .to_owned();
    assert_eq!(population.try_into(), Ok(6u64));
    // or
    assert_eq!(
        u64::try_from(
            world
                .globals
                .get("population")
                .expect("no population")
                .to_owned()
        ),
        Ok(6)
    );

    assert!(world.output[0].contains("Setup complete"));
}
