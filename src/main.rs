use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// CLI tool for retrieving configuration values from any source
struct Args {
    /// List of keys to retrieve
    key_list: String,

    /// Which source config file to use
    #[arg(short, long)]
    source: String,

    /// Output format
    #[arg(short, long, default_value_t = String::from("dotenv"))]
    format: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Source {
    Cmd,
    Value,
}

#[derive(Debug, Deserialize)]
struct ConfigValueSource {
    source: Source,
    exec: Option<String>,
    args: Option<Vec<String>>,
    value: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let input_path = args.source;
    let input = match parse_config(&input_path) {
        Ok(map) => map,
        Err(error) => {
            return Err(error.into());
        }
    };
    let mut map = HashMap::new();
    for key in args.key_list.split(",") {
        if let Some(config) = input.get(key) {
            let value = get_config_value(config)?;
            map.insert(key, value);
        } else {
            return Err(format!("Key '{}' not found in source config", key).into());
        }
    }

    let output = match args.format.as_str() {
        "json" => output_json(&map)?,
        "dotenv" => output_dotenv(&map)?,
        _ => output_dotenv(&map)?,
    };
    print!("{}", output);
    Ok(())
}

fn get_config_value(config: &ConfigValueSource) -> Result<String, Box<dyn std::error::Error>> {
    match config.source {
        Source::Cmd => {
            let mut cmd = std::process::Command::new(config.exec.as_ref().unwrap());
            cmd.args(config.args.as_ref().unwrap_or(&Vec::new()));
            let output = cmd.output()?;
            if !output.stderr.is_empty() {
                return Err(format!(
                    "Error running command: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
                .into());
            }
            let value = String::from_utf8(output.stdout)?;
            return Ok(value);
        }
        Source::Value => {
            return Ok(config.value.as_ref().unwrap_or(&"".to_string()).to_string());
        }
    }
}

fn parse_config(
    path: &str,
) -> Result<HashMap<String, ConfigValueSource>, Box<dyn std::error::Error>> {
    let input_file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return Err(error.into());
        }
    };
    let reader = BufReader::new(input_file);
    let input: HashMap<String, ConfigValueSource> = serde_json::from_reader(reader)?;
    Ok(input)
}

fn output_json(map: &HashMap<&str, String>) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_string(&map)?;
    Ok(json)
}

fn output_dotenv(map: &HashMap<&str, String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::new();
    for (key, value) in map.iter() {
        result.push_str(&format!("{}={}\n", key, value));
    }
    Ok(result)
}
