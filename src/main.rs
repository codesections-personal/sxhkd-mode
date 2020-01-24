use clap::{crate_name, crate_version, App, Arg};
use config;
use regex::Regex;
use std::collections::HashMap;
use utils::{dependencies, die_with_msg, sh};

fn main() {
    let matches = App::new(crate_name!())
        .about(
            "Check the provided MODE against the rules listed in the configuration file.  \
             If a match is found, set the current sxhkd mode to the mode specified in that rule.",
        )
        .version(crate_version!())
        .arg(
            Arg::with_name("MODE")
                .required(true)
                .help("The new sxhkd mode"),
        )
        .get_matches();
    dependencies(crate_name!(), &["ln", "pkill"]);

    let mut settings = config::Config::new();
    settings
        .merge(config::File::with_name(
            "/home/dsock/.config/sxhkd/modes.toml",
        ))
        .unwrap_or_else(|_| die_with_msg("Invalid configuration file"));
    let settings = settings.try_into::<HashMap<String, String>>().unwrap();

    let mode = matches.value_of("MODE").expect("required");

    let target_mode = settings
        .iter()
        .fold(String::from("empty_mode"), |cur, (key, value)| {
            let re = Regex::new(value).unwrap();
            match re.is_match(mode) {
                true => String::from(key),
                false => cur,
            }
        });

    foo();

    sh(&format!(
        r#"ln --symbolic --no-dereference --force {sxhkd_dir}/{mode} {sxhkd_dir}/current_mode"#,
        sxhkd_dir = "/home/dsock/.config/sxhkd",
        mode = target_mode
    ));
    sh("pkill -USR1 -x sxhkd");
}

fn foo() -> () {
    println!("bar");
}
