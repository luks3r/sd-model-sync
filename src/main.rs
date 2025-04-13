mod argparser;
mod configuration;
mod link;

use std::path::PathBuf;

use argparser::ArgumentParser;
use configuration::{ComfyUIConfig, Config, FolderStructure, GeneralConfig, WebUIConfig};

use log::{debug, LevelFilter};

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
    let mut argparser = ArgumentParser::new();
    argparser.set_program_name("sd-model-sync");
    argparser.set_program_description("Sync Stable Diffusion models between different folders");

    argparser.add_positional("general", Some("Path to general models directory"));
    argparser.add_optional(
        "--config",
        Some("-c"),
        Some("Optional path to configuration file"),
    )?;
    argparser.add_optional(
        "--comfyui",
        Some("-cf"),
        Some("Optional path to ComfyUI models directory"),
    )?;
    argparser.add_optional(
        "--webui",
        Some("-w"),
        Some("Optional path to WebUI models directory"),
    )?;
    argparser.add_optional(
        "--verbosity",
        Some("-v"),
        Some("Increase verbosity of the logger"),
    )?;

    argparser.parse()?;

    let general_path: PathBuf = argparser.positional("general")?.into();

    let config: Option<Config> = argparser.optional("-c").ok().map(|path| {
        let config_data = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&config_data).unwrap()
    });

    let comfyui_path = if let Some(c) = config.as_ref() {
        Some(config.as_ref().unwrap().comfyui.path.clone())
    } else {
        argparser.optional("-cf").ok().map(PathBuf::from)
    };

    let webui_path = if let Some(c) = config.as_ref() {
        Some(config.as_ref().unwrap().webui.path.clone())
    } else {
        argparser.optional("-w").ok().map(PathBuf::from)
    };

    let verbosity = argparser
        .optional("-v")
        .unwrap_or_else(|_| String::from("0"))
        .parse::<u8>()?;

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
