extern crate clap;
extern crate dirs;
extern crate reqwest;
#[macro_use] extern crate serde_json;

use clap::{App, Arg, SubCommand};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::num::ParseIntError;
use std::process;
use url::{Url, ParseError};

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

    let home_dir_path = dirs::home_dir().unwrap();
    let mut config_file = home_dir_path.into_os_string().into_string().unwrap();


    if let Some(matches) = matches.subcommand_matches("init") {
        let mut path_str = "/.irgsh";
        config_file.push_str(&path_str);
        fs::create_dir(&config_file).ok();
        path_str = "/IRGSH_CHIEF_ADDRESS";
        config_file.push_str(&path_str);
        // TOOD validate URL
        let url = matches.value_of("chief").unwrap();
        let key = matches.value_of("key").unwrap();
        println!("{url}");
        println!("{key}");
        let mut f = File::create(&config_file).expect("Unable to create file");
        f.write_all(url.as_bytes()).expect("Unable to write data");
        println!(
            "Successfully sets the chief address to {}. Now you can use irgsh-cli.",
            matches.value_of("chief").unwrap()
        );
        return Ok(());
    }

    config_file.push_str("/.irgsh/IRGSH_CHIEF_ADDRESS");
    let chief_url_result = fs::read_to_string(&config_file);
    let chief_url = match chief_url_result {
        Ok(url) => url,
        Err(error) => "Error: unable to read config file. Please initialize the irgsh-cli first. See --help for further information.".to_string(),
    };

    if chief_url.contains("Error") {
       return graceful_exit(chief_url, 1);
    }

    if let Some(matches) = matches.subcommand_matches("submit") {
        println!("Chief       : {}", chief_url);
				if matches.value_of("source").unwrap().chars().count() > 0 {
        	println!("Source URL  : {}", matches.value_of("source").unwrap());
				}
				if matches.value_of("package").unwrap().chars().count() > 0 {
        	println!("Package URL : {}", matches.value_of("package").unwrap());
				}
    		let mut chief_url = fs::read_to_string(&config_file).expect("Unable to read config file. Please initialize the irgsh-cli first. See --help for further information.");
        chief_url.push_str("/api/v1/submit");
        println!("Submit URL  : {}", chief_url);

    		let payload: serde_json::Value = json!({
            "sourceUrl": matches.value_of("source").unwrap(),
            "packageUrl": matches.value_of("package").unwrap()
    		});

				let _result: serde_json::Value = match post(chief_url, payload) {
					Ok(result) => result,
					Err(_e) => return Ok(())
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
    		    .json(&json_payload
    		    )
    		    .send()?
            .json()?;

          println!("{:#?}", echo_json);
          return Ok(echo_json)
}

fn handler(e: reqwest::Error) {
   if e.is_http() {
       match e.url() {
           None => println!("No Url given"),
           Some(url) => println!("Problem making request to: {}", url),
       }
   }
   // Inspect the internal error and output it
   if e.is_serialization() {
      let serde_error = match e.get_ref() {
           None => return,
           Some(err) => err,
       };
       println!("problem parsing information {}", serde_error);
   }
   if e.is_redirect() {
       println!("server redirecting too many times or making loop");
   }
}
