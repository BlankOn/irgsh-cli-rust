extern crate clap;
extern crate dirs;
extern crate reqwest;
#[macro_use]
extern crate serde_json;

use clap::{App, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use std::fs;
use std::num::ParseIntError;
use std::path::Path;
use std::process;

extern crate question;
use question::{Answer, Question};


const CONFIG_FILE_PATH: &str = ".irgsh/irgsh.yaml";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    chief_url: String,
    maintainer_key: String,
    config_file_path: String,
}

impl Config {
    fn set(&self) {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.config_file_path)
            .expect("Couldn't write file");
        serde_yaml::to_writer(f, &self).unwrap();
    }
    fn load(&self) -> Config {
        let f = std::fs::File::open(&self.config_file_path).expect("Could not open file.");
        let loaded: Config = serde_yaml::from_reader(f).expect("Could not read values.");
        return loaded;
    }
}

fn graceful_exit<E, T>(message: String, exit_code: i32) -> Result<T, E> {
    println!("{}", message);
    process::exit(exit_code);
}

fn main() -> Result<(), ParseIntError> {
    let author =
        "BlankOn Developer <blankon-dev@googlegroups.com>\nHerpiko Dwi Aguno <herpiko@aguno.xyz>";
    let version = "0.0.1";

    let matches = App::new("irgsh-cli")
    								.version(version)
    								.author(author)
    								.about("IRGSH command line interface")
    								.subcommand(SubCommand::with_name("init")
    									.about("Initializes the command line interface program for the first time. You need the IRGSH chief URL address.")
    									.version(version)
    									.author(author)
    									.arg(Arg::with_name("chief")
    										.short("chief")
    										.long("chief")
    										.value_name("URL")
    										.help("Sets the IRGSH Chief address")
                                            .required(true)
    										.takes_value(true))
    									.arg(Arg::with_name("key")
    										.short("key")
    										.long("key")
    										.value_name("key")
    										.help("Sets the maintainer key")
                                            .required(true)
    										.takes_value(true))
                                            )
    								.subcommand(SubCommand::with_name("submit")
    									.about("Submits the package and source (optional).")
    									.version(version)
    									.author(author)
    									.arg(Arg::with_name("package")
    										.short("-p")
    										.long("package")
    										.value_name("URL")
    										.help("Package Git URL")
                        .required(true)
    										.takes_value(true))
    									.arg(Arg::with_name("source")
    										.short("-s")
    										.long("source")
    										.value_name("URL")
    										.help("Source Git URL")
                        .required(true)
    										.takes_value(true)))
    								.subcommand(SubCommand::with_name("status")
    									.about("Checks the status of a pipeline.")
    									.version(version)
    									.author(author)
    									.arg(Arg::with_name("PIPELINE_ID")
                        .help("Pipeline ID")
                        .required(true)
                        .index(1)))
    								.subcommand(SubCommand::with_name("status")
    									.about("Watch the latest log of the pipeline in real time.")
    									.version(version)
    									.author(author)
    									.arg(Arg::with_name("PIPELINE_ID")
                        .help("Pipeline ID")
                        .required(true)
                        .index(1)))
    								.get_matches();

    // Prepare path
    let home_dir_path = dirs::home_dir().unwrap();
    let home_dir_path = home_dir_path.into_os_string().into_string().unwrap();
    let mut config_file_path = home_dir_path.clone();
    config_file_path.push_str("/");
    config_file_path.push_str(&CONFIG_FILE_PATH);

    // Global config, could be reused anywhere under subcommand scopes bellow
    let mut config = Config {
        config_file_path: config_file_path.to_string(),
        chief_url: "".to_string(),
        maintainer_key: "".to_string(),
    };

    if let Some(matches) = matches.subcommand_matches("init") {
        // Retrieve the arguments/params
        let url = matches.value_of("chief").unwrap();
        let key = matches.value_of("key").unwrap();
        config.chief_url = url.to_string();
        config.maintainer_key = key.to_string();

        if Path::new(&config.config_file_path).exists() {
            let loaded = config.load();
            println!("Current config file is already exist with this content:");
            println!("{:?}", loaded);
            let answer = Question::new("Do you want to continue to override this with your new config?")
                .default(Answer::YES)
                .show_defaults()
                .confirm();
            if answer != Answer::YES {
                return graceful_exit("".to_string(), 0);
            }
        }

        let mut config_dir = home_dir_path.clone();
        config_dir.push_str("/.irgsh");
        fs::create_dir(&config_dir).ok();

        // Write the config to file
        config.set();
        println!(
            "Successfully sets the chief address to {}. Now you can use irgsh-cli. Happy Hacking",
            url.to_string()
        );
        return Ok(());
    }

    if !Path::new(&config.config_file_path).exists() {
        let err_message: &str = "Error: unable to read config file. Please initialize the irgsh-cli first. See --help for further information.";
        return graceful_exit(err_message.to_string(), 1);
    }

    if let Some(matches) = matches.subcommand_matches("submit") {
        // Retrieve the arguments/params
        let source = matches.value_of("source").unwrap();
        let package = matches.value_of("package").unwrap();

        config.load();

        println!("Chief       : {}", &config.chief_url);
        if matches.value_of("source").unwrap().chars().count() > 0 {
            println!("Source URL  : {}", source);
        }
        if matches.value_of("package").unwrap().chars().count() > 0 {
            println!("Package URL : {}", package);
        }
        let mut chief_url = config.chief_url.clone();
        chief_url.push_str("/api/v1/submit");
        println!("Submit URL  : {}", chief_url);

        let payload: serde_json::Value = json!({
        "sourceUrl": source,
        "packageUrl": package
        });

        let _result: serde_json::Value = match post(chief_url, payload) {
            Ok(result) => result,
            Err(_e) => return Ok(()),
        };

        return Ok(());
    } else if let Some(matches) = matches.subcommand_matches("status") {
        println!(
            "Status of PipelineID: {}",
            matches.value_of("PIPELINE_ID").unwrap()
        );
        return Ok(());
    } else if let Some(matches) = matches.subcommand_matches("watch") {
        println!(
            "Watching PipelineID: {}",
            matches.value_of("PIPELINE_ID").unwrap()
        );
        return Ok(());
    // Fall back to status subcommand if the pipeline was done.
    } else {
        println!("\nPlease run by a subcommand. See --help for further information.");
        return Ok(());
    }
}

fn post(url: String, json_payload: serde_json::Value) -> Result<serde_json::Value, reqwest::Error> {
    let echo_json: serde_json::Value = reqwest::Client::new()
        .post(&url)
        .json(&json_payload)
        .send()?
        .json()?;

    println!("{:#?}", echo_json);
    return Ok(echo_json);
}
