mod civitai;
mod configuration;
mod hash;
mod link;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Seek, SeekFrom};
use std::{path::Path, path::PathBuf, process::exit};

use configuration::{ComfyUIConfig, Config, FolderStructure, GeneralConfig, WebUIConfig};

use civitai::{query_model_info, ModelInfo, ModelType};
use hash::EldenRing;
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

fn get_model_info<P: AsRef<Path>>(model: P, cache_path: Option<P>) -> Result<ModelInfo, Box<dyn std::error::Error>> {
    let model_path = model.as_ref().to_path_buf();
    let model_path_string = model_path.to_string_lossy().to_string();

    let cache_path = match cache_path {
        Some(path) => path.as_ref().to_path_buf(),
        None => PathBuf::from("cache.json"),
    };

    let mut cach_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(cache_path)?;

    let mut data: HashMap<String, String> = {
        let reader = BufReader::new(&cach_file);
        serde_json::from_reader(reader).unwrap_or_default()
    };

    let hash = if let Entry::Vacant(entry) = data.entry(model_path_string.clone()) {
        let new_hash = EldenRing::from_file(&model_path)?;
        entry.insert(new_hash.clone());
        new_hash
    } else {
        data.get(&model_path_string).cloned().unwrap_or_default()
    };

    cach_file.set_len(0)?;
    cach_file.seek(SeekFrom::Start(0))?;

    let writer = BufWriter::new(&cach_file);
    serde_json::to_writer_pretty(writer, &data)?;

    Ok(query_model_info(&hash)?)
}

fn adopt_orphan(orphan: PathBuf, home: PathBuf, parents: ModelType, nationality: String) -> Result<(), Box<dyn std::error::Error>> {
    let new_path = home
        .join(parents.general_directory())
        .join(nationality)
        .join(orphan.file_name().unwrap());

    debug!(
        "Adopting orphan checkpoint: {} to {}",
        orphan.display(),
        new_path.display()
    );
    std::fs::rename(&orphan, &new_path)?;
    Ok(())
}

fn sort_models(root_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let orphaned_models: Vec<PathBuf> = root_path
        .read_dir()?
        .filter_map(|dir_entry| match dir_entry {
            Ok(file) => {
                let path = file.path();
                if !path.is_dir() {
                    if ["safetensors", "ckpt", "pt", "pth", "bin"].contains(
                        &path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    ) {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(err) => {
                debug!("Error reading directory: {}", err);
                None
            }
        })
        .collect();

    orphaned_models.iter().for_each(|path| {
        debug!("Orphaned model: {}", path.display());
        match get_model_info(path, Some(&root_path.join("orphan_cache.json"))) {
            Ok(info) => {
                let model_type = info.model_info.model_type;
                let base_model = info.base_model;
                adopt_orphan(
                    path.to_path_buf(),
                    root_path.clone(),
                    model_type,
                    base_model,
                )
                .unwrap();
            }
            Err(err) => {
                debug!("Error getting model info: {}", err);
            }
        }
    });

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

    let general_path = parsed_args.general.canonicalize()?;

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
