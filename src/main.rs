use clap::{crate_version, App, Arg};
use std::str::FromStr;
use strum;
use strum_macros::{Display, EnumString};

fn main() {
    let matches = App::new("sxhkd-mode")
        .version(crate_version!())
        .arg(
            Arg::with_name("MODE")
                .required(true)
                .possible_values(&["Emacs", "Fundamental", "APL"])
                .help("The sxhkd mode to set"),
        )
        .get_matches();

    println!(
        "Hello, world? mode: {}",
        matches.value_of("MODE").expect("required")
    );
}
