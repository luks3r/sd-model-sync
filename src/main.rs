mod configuration;
mod link;

use std::{path::PathBuf, process::exit};

use configuration::{ComfyUIConfig, Config, FolderStructure, GeneralConfig, WebUIConfig};

use log::{debug, LevelFilter};
use structopt::StructOpt;

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

fn process_comfyui(
    models_structure: &FolderStructure,
    config: &Option<Config>,
    comfyui_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = comfyui_path {
        debug!("Linking to ComfyUI: {:?}", path);

        let comfyui_structure: FolderStructure = match config {
            Some(config) => config.clone().comfyui.try_into()?,
            None => ComfyUIConfig::new(path).try_into()?,
        };

        models_structure.soft_link_to(&comfyui_structure)?;
    }

    Ok(())
}

fn process_webui(models_structure: &FolderStructure, config: &Option<Config>, webui_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = webui_path {
        debug!("Linking to WebUI: {:?}", path);

        let webui_structure: FolderStructure = match config {
            Some(config) => config.clone().webui.try_into()?,
            None => WebUIConfig::new(path).try_into()?,
        };

        models_structure.soft_link_to(&webui_structure)?;
    }

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

    let general_path = parsed_args.general;

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

    let models_structure: FolderStructure = GeneralConfig::new(general_path).into();

    process_comfyui(&models_structure, &config, comfyui_path)?;
    process_webui(&models_structure, &config, webui_path)?;

    Ok(())
}
