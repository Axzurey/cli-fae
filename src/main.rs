use std::{fs::File, io::BufReader, collections::HashMap};
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CosyConfig {
    main: Option<String>,
    args: Option<Vec<String>>,
    scripts: Option<std::collections::HashMap<String, String>>,
    language: Option<String>,
    append_output_for_consecutive_runs: Option<bool>,
    send_output_to_file: Option<String>,
}

struct LanguageInformation {
    get_string: Box<dyn Fn(String) -> String>
}

fn main() {
    let language_maps: HashMap<&str, LanguageInformation> = HashMap::from([
        ("py", LanguageInformation { get_string: Box::new(|file_path| format!("python {file_path}"))}),
        ("py3", LanguageInformation { get_string: Box::new(|file_path| format!("python3 {file_path}"))}),
        ("python", LanguageInformation { get_string: Box::new(|file_path| format!("python {file_path}"))}),
        ("python3", LanguageInformation { get_string: Box::new(|file_path| format!("python3 {file_path}"))}),
        ("node", LanguageInformation { get_string: Box::new(|file_path| format!("node {file_path}"))}),
        ("nodejs", LanguageInformation { get_string: Box::new(|file_path| format!("node {file_path}"))}),
    ]);


    let command = std::env::args().nth(1).expect("You have not provided a command to execute.");

    match command.as_str() {
        "start" => {
            let mut config_path = std::env::current_dir().unwrap();
            config_path.push("cosy.json"); //the file path

            if !config_path.exists() || config_path.is_dir() {
                panic!("There exists no cosy.json file in this directory.")
            }

            let file = File::open(config_path).expect("Could not read cosy.json file.");

            let reader = BufReader::new(file);

            let config: CosyConfig = serde_json::from_reader(reader).expect("Could not read cosy.json. It may be malformed.");
            
            let current_directory = std::env::current_dir().unwrap().to_str().unwrap().to_owned();

            let main_file_arg = config.main.expect("The 'main' key must be explicitly set because this will be the entry point.");

            let main_file_path = current_directory + &format!("/{main_file_arg}");

            let la = config.language.expect("The 'lanuage' key must be explicitly set!").to_lowercase();
            let language = la.trim();

            let language_information = match language_maps.get(&language) {
                Some(i) => i,
                _ => panic!("{}", format!("{language} is not supported."))
            };

            let mut command_transformed = (language_information.get_string)(main_file_path);

            if config.send_output_to_file.is_some() {
                let override_file_contents = match config.append_output_for_consecutive_runs {
                    Some(o) => o,
                    _ => false,
                };

                let time_capture_regex = Regex::new(r"#t").unwrap(); //maybe just use str.replace
            }

        },
        _ => {
            panic!("{}", format!("{command} is not a valid command!"))
        }
    }
}