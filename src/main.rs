use std::{fs::{File, self}, io::BufReader, collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Local;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FaeConfig {
    main: Option<String>,
    args: Option<Vec<String>>,
    scripts: Option<std::collections::HashMap<String, String>>,
    language: Option<String>,
    append_output_for_consecutive_runs: Option<bool>,
    send_output_to_file: Option<String>,
    shell: Option<String>, //cmd, psh
    external_dependencies: Option<std::collections::HashMap<String, String>>,
    installation_command: Option<String>
}

struct LanguageInformation {
    get_string: Box<dyn Fn(String) -> String>
}

fn write_to_config(config: FaeConfig) {

    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    config.serialize(&mut ser).unwrap();

    fs::write("fae.config.json", String::from_utf8(ser.into_inner()).unwrap()).expect("Unable to write to config file");
}

fn get_fae_config() -> FaeConfig {
    let mut config_path = std::env::current_dir().unwrap();
    config_path.push("fae.config.json"); //the file path

    if !config_path.exists() || config_path.is_dir() {
        panic!("There exists no fae.json file in this directory.")
    }

    let file = File::open(config_path).expect("Could not read fae.json file.");

    let reader = BufReader::new(file);

    let config: FaeConfig = serde_json::from_reader(reader).expect("Could not read fae.json. It may be malformed.");

    return config;
}

fn get_shell_type() -> String {
    let config = get_fae_config();

    let shell_selection = match config.shell {
        Some(e) => e,
        None => "cmd".to_string(),
    };

    let shell_map: HashMap<&str, &str> = HashMap::from([
        ("psh", "powershell"),
        ("powershell", "powershell"),
        ("cmd", "cmd"),
        ("command", "cmd")
    ]);

    if !shell_map.contains_key(shell_selection.as_str()) {
        panic!("{shell_selection} is not a supported shell type!");
    }
    return shell_map.get(shell_selection.as_str()).unwrap().to_string();
}

fn main() {
    let language_maps: HashMap<&str, LanguageInformation> = HashMap::from([
        ("py", LanguageInformation { get_string: Box::new(|file_path| format!("python \"{file_path}\""))}),
        ("py3", LanguageInformation { get_string: Box::new(|file_path| format!("python3 \"{file_path}\""))}),
        ("python", LanguageInformation { get_string: Box::new(|file_path| format!("python \"{file_path}\""))}),
        ("python3", LanguageInformation { get_string: Box::new(|file_path| format!("python3 \"{file_path}\""))}),
        ("node", LanguageInformation { get_string: Box::new(|file_path| format!("node \"{file_path}\""))}),
        ("nodejs", LanguageInformation { get_string: Box::new(|file_path| format!("node \"{file_path}\""))}),
    ]);


    let command = std::env::args().nth(1).expect("You have not provided a command to execute.");

    match command.as_str() {
        "install" => {
            let mut config = get_fae_config();

            let mut dependencies = match config.external_dependencies {
                Some(d) => d,
                _ => HashMap::new()
            }; //Name, Version

            let package = std::env::args().nth(2).expect("A valid package was not provided.");

            let package_version = match std::env::args().nth(3) {
                Some(p) => p,
                _ => "@latest".to_string()
            };


            let shell = get_shell_type();

            let mut cmd = std::process::Command::new(shell);
        
            let mut command = config.installation_command.expect("installationCommand is not set in fae.config.json file");

            command = command.replace("cosy.pkg", &package).replace("cosy.version", &package_version);
            
            let mut args: Vec<String> = [
                
            ].into();

            dependencies.insert(package, package_version);

            config.external_dependencies = Some(dependencies);

            write_to_config(config);
            
        },
        "start" => {

            let config = get_fae_config();

            let main_file_arg = config.main.expect("The 'main' key must be explicitly set because this will be the entry point.");

            let main_file_path = std::env::current_dir().unwrap().join(PathBuf::from(main_file_arg)).to_str().unwrap().to_owned();

            let la = config.language.expect("The 'lanuage' key must be explicitly set!").to_lowercase();
            let language = la.trim();

            let language_information = match language_maps.get(&language) {
                Some(i) => i,
                _ => panic!("{}", format!("{language} is not supported."))
            };

            let command_transformed = (language_information.get_string)(main_file_path);

            let shell = get_shell_type();

            let mut cmd = std::process::Command::new(shell);
            
            let spl = command_transformed.split_once(" ").expect("unable to parse file path");

            let mut args: Vec<String> = [
                spl.0.to_owned(),
                spl.1.to_owned(),
            ].into();

            if config.send_output_to_file.is_some() {
                let override_file_contents = match config.append_output_for_consecutive_runs {
                    Some(o) => !o,
                    _ => true,
                };

                let system_time = Local::now().format("%Y-%m-%d@%Hh%Mm%Ss").to_string();

                let output_file_name = std::env::current_dir().unwrap().join(PathBuf::from(&config.send_output_to_file.unwrap().replace("@fae.time", &system_time))).to_str().unwrap().to_owned();
            
                if override_file_contents {
                    args.push(">".to_owned());
                    args.push(format!("\"{}\"", output_file_name));
                }
                else {
                    args.push(">>".to_owned());
                    args.push(format!("\"{}\"", output_file_name));
                }
            }

            cmd.args(&args);

            cmd.spawn().expect("Unable to spawn a terminal...");
        },
        _ => {
            panic!("{}", format!("{command} is not a valid command!"))
        }
    }
}