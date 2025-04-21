mod api;
mod civitai;
mod configuration;
mod hash;
mod link;

use std::path::PathBuf;
use std::process::exit;

use log::debug;
use log::info;
use log::LevelFilter;
use structopt::StructOpt;

use crate::api::process_comfyui;
use crate::api::process_webui;
use crate::api::sort_models;
use crate::configuration::Config;
use crate::configuration::FolderStructure;
use crate::configuration::GeneralConfig;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "model_sync",
    about = "Sync models between general directory and ComfyUI or WebUI"
)]
struct Args {
    /// Path to general models directory
    #[structopt(parse(from_os_str))]
    general: PathBuf,

    /// Set logging verbosity level
    #[structopt(short, long, default_value = "0")]
    verbosity: u8,

    /// Optional path to config file
    #[structopt(short, long)]
    toml_config: Option<PathBuf>,

    /// Optional path to comfyui models directory
    #[structopt(short, long)]
    comfyui: Option<PathBuf>,

    /// Optional path to webui models directory
    #[structopt(short, long)]
    webui: Option<PathBuf>,
}

fn setup_logger(verbosity: u8) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .format_target(false)
        .init();

    debug!("Logger initialized with level: {}", log_level);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Option<Args> = match Args::from_args_safe() {
        Ok(args) => Some(args),
        Err(err) => {
            match err.kind {
                structopt::clap::ErrorKind::HelpDisplayed => println!("{}", err.message),
                structopt::clap::ErrorKind::VersionDisplayed => println!("{}", err.message),
                _ => println!("{}", err),
            }
            None
        }
    };

    let Some(parsed_args) = args else {
        exit(0);
    };

    let general_path = parsed_args.general.canonicalize()?;
    info!("General path: {}", general_path.display());

    let config: Option<Config> = parsed_args.toml_config.map(|path| {
        let config_data = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&config_data).unwrap()
    });

    let comfyui_path = if let Some(c) = config.as_ref() {
        Some(c.comfyui.path.clone())
    } else {
        parsed_args.comfyui
    };

    let webui_path = if let Some(c) = config.as_ref() {
        Some(c.webui.path.clone())
    } else {
        parsed_args.webui
    };

    let verbosity = parsed_args.verbosity;

    if comfyui_path.is_none() && webui_path.is_none() && config.is_none() {
        return Err("No paths provided".into());
    }

    setup_logger(verbosity)?;

    if let Some(cfg) = &config {
        debug!("Current config: {:?}", cfg);
    }

    sort_models(general_path.clone())?;

    let models_structure: FolderStructure = GeneralConfig::new(general_path).into();

    process_comfyui(&models_structure, &config, comfyui_path)?;
    process_webui(&models_structure, &config, webui_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::civitai::ModelInfo;
    use crate::civitai::API_URL;
    use crate::hash::EldenRing;

    #[test]
    fn test_eldenring_hash() {
        let dummy_data: [u8; 1024] = [7; 1024];
        let dummy_reader = BufReader::new(dummy_data.as_slice());
        let hash = EldenRing::calculate_hash_sha256(dummy_reader);
        assert!(hash.is_ok());
    }

    #[test]
    fn test_civitai_query() {
        let example_hash = "3D3766E175328CDF5B23287E5D662BFCD905E97F91CC09DE3AEE6B442341F906";
        let url = format!("{}{}", API_URL, example_hash);

        let first_response = reqwest::blocking::get(&url);
        assert!(first_response.is_ok(), "couldn't get response");
        let first_response = first_response.unwrap();

        let mut response_text: Result<String, reqwest::Error> = Ok(String::default());

        match first_response.status() {
            reqwest::StatusCode::OK => {
                response_text = first_response.text();
            }
            reqwest::StatusCode::NOT_FOUND => {
                println!("Warning: model not found");
            }
            reqwest::StatusCode::SERVICE_UNAVAILABLE => {
                println!("Warning: service unavailable");
            }
            _ => {
                println!("Warning: couldn't get response");
            }
        }

        assert!(response_text.is_ok(), "couldn't get response text");
        let text = response_text.unwrap();
        assert!(!text.is_empty(), "response text is empty");

        let json_from_response_text: Result<ModelInfo, serde_json::Error> = serde_json::from_str(&text);
        assert!(
            json_from_response_text.is_ok(),
            "couldn't parse response into JSON: {}",
            json_from_response_text.err().unwrap()
        );
    }
}
