use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

use log::debug;
use log::error;
use log::info;

use crate::civitai::query_model_info;
use crate::civitai::ModelInfo;
use crate::civitai::ModelType;
use crate::configuration::ComfyUIConfig;
use crate::configuration::Config;
use crate::configuration::FolderStructure;
use crate::configuration::WebUIConfig;
use crate::hash::EldenRing;

#[derive(Debug)]
pub enum APIError {
    ModelNotFound(String),
    SerdeJson(String),
    EldenError(String),
    CivitAiError(String),
    Io(String),
    Unspecified(String),
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIError::ModelNotFound(msg) => write!(f, "Model not found error: {}", msg),
            APIError::SerdeJson(msg) => write!(f, "Serde JSON error: {}", msg),
            APIError::EldenError(msg) => write!(f, "Elden error: {}", msg),
            APIError::CivitAiError(msg) => write!(f, "CivitAI error: {}", msg),
            APIError::Io(msg) => write!(f, "IO error: {}", msg),
            APIError::Unspecified(msg) => write!(f, "Unspecified error: {}", msg),
        }
    }
}

impl From<std::io::Error> for APIError {
    fn from(_: std::io::Error) -> Self {
        APIError::Io("IO error".to_string())
    }
}

impl From<&str> for APIError {
    fn from(msg: &str) -> Self {
        APIError::ModelNotFound(msg.to_string())
    }
}

impl From<serde_json::Error> for APIError {
    fn from(err: serde_json::Error) -> Self {
        APIError::SerdeJson(err.to_string())
    }
}

impl From<crate::hash::EldenError> for APIError {
    fn from(err: crate::hash::EldenError) -> Self {
        APIError::EldenError(err.to_string())
    }
}

impl From<crate::civitai::CivitAiError> for APIError {
    fn from(err: crate::civitai::CivitAiError) -> Self {
        APIError::CivitAiError(err.to_string())
    }
}

impl std::error::Error for APIError {}

type Result<T> = std::result::Result<T, APIError>;

pub fn lookup_cached_model_hash<P: AsRef<Path>>(model: P, cache_json_path: P) -> Result<String> {
    let model_path_string = model.as_ref().to_string_lossy().to_string();
    let cache_path = cache_json_path.as_ref().to_path_buf();
    let cache_file = OpenOptions::new().read(true).open(cache_path)?;
    debug!("Looking for cached hash for {}", model_path_string);

    let data: HashMap<String, String> = {
        let reader = BufReader::new(&cache_file);
        serde_json::from_reader(reader).unwrap_or_default()
    };

    let result = data.get(&model_path_string);

    match result {
        Some(hash) => Ok(hash.to_string()),
        None => Err(APIError::ModelNotFound(
            "Model not found in cache".to_string(),
        )),
    }
}

pub fn cache_model_hash<P: AsRef<Path>>(hash: &str, model: P, json_path: P) -> Result<()> {
    let model_path = model.as_ref().to_path_buf();
    let model_path_string = model_path.to_string_lossy().to_string();
    let cache_path = json_path.as_ref().to_path_buf();

    let mut cache_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(cache_path)?;

    let mut data: HashMap<String, String> = {
        let reader = BufReader::new(&cache_file);
        serde_json::from_reader(reader).unwrap_or_default()
    };

    if let Entry::Vacant(entry) = data.entry(model_path_string.clone()) {
        debug!("Caching hash for {}", &model_path_string);
        entry.insert(hash.to_string());
    }

    cache_file.set_len(0)?;
    cache_file.seek(SeekFrom::Start(0))?;

    let writer = BufWriter::new(&cache_file);
    serde_json::to_writer_pretty(writer, &data)?;

    Ok(())
}

pub fn get_model_info<P: AsRef<Path>>(model: P, cache_json_path: Option<P>) -> Result<ModelInfo> {
    let model_path = model.as_ref().to_path_buf();
    let cache_path = match cache_json_path {
        Some(path) => path.as_ref().to_path_buf(),
        None => PathBuf::from("cache.json"),
    };
    debug!("Getting model info for {}", model_path.display());

    let hash = match lookup_cached_model_hash(&model_path, &cache_path) {
        Ok(hash) => {
            debug!("Using cached hash for {}", model_path.display());
            hash
        }
        Err(_) => {
            info!("Calculating hash for {}", model_path.display());
            let hash = EldenRing::from_file(&model)?;
            cache_model_hash(&hash, &model_path, &cache_path)?;
            hash
        }
    };

    let model_info = query_model_info(&hash)?;

    Ok(model_info)
}

pub fn move_orphan_model<P: AsRef<Path>>(orphan_model: P, destination: P, model_type: ModelType, base_model: &str) -> Result<()> {
    let orphan_model_path = orphan_model.as_ref().to_path_buf();
    let destination_path = destination.as_ref().to_path_buf();
    let model_type_name = model_type.general_directory();
    let base_model_name = base_model.to_lowercase();
    let Some(file_name) = orphan_model_path.file_name() else {
        return Err("Error getting file name".into());
    };

    let new_path = destination_path
        .join(model_type_name)
        .join(base_model_name)
        .join(file_name);

    let new_parent = new_path.parent().unwrap_or(&destination_path);

    info!(
        "Moving orphan model {} to {}",
        orphan_model_path.display(),
        new_path.display()
    );

    if !new_parent.exists() {
        debug!("Creating directory {}", new_parent.display());
        std::fs::create_dir_all(new_parent)?;
    }

    std::fs::rename(&orphan_model_path, &new_path)?;
    Ok(())
}

pub fn get_orphan_models<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>> {
    let root_path = root.as_ref().to_path_buf();
    let Ok(read_dir) = root_path.read_dir() else {
        return Err("Error reading directory".into());
    };

    let mut dir_entries: Vec<DirEntry> = vec![];

    for entry in read_dir {
        let Ok(entry) = entry else {
            continue;
        };
        dir_entries.push(entry);
    }

    let orphan_model_entries: Vec<&DirEntry> = dir_entries
        .iter()
        .filter(|dir_entry| {
            let allowed_extensions = ["safetensors", "ckpt", "pt", "pth", "bin"];
            let path = dir_entry.path();
            if !path.is_dir() {
                allowed_extensions.contains(
                    &path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                )
            } else {
                false
            }
        })
        .collect();

    let orphan_model_paths: Vec<PathBuf> = orphan_model_entries
        .iter()
        .map(|path| path.path().to_path_buf())
        .collect();

    Ok(orphan_model_paths)
}

pub fn sort_models<P: AsRef<Path>>(root: P) -> Result<()> {
    let root_path = root.as_ref().to_path_buf();
    let orphan_models = get_orphan_models(&root_path)?;
    orphan_models.iter().for_each(
        |path| match get_model_info(path, Some(&root_path.join("orphan_cache.json"))) {
            Ok(info) => {
                let model_type = info.model_info.model_type;
                let base_model = info.base_model.unwrap_or("Other".to_string());
                match move_orphan_model(
                    path.to_path_buf(),
                    root_path.clone(),
                    model_type,
                    &base_model,
                ) {
                    Ok(_) => (),
                    Err(err) => error!("Error moving orphan model: {}", err),
                }
            }
            Err(err) => error!("Error getting model info: {}", err),
        },
    );

    Ok(())
}

pub fn process_comfyui(models_structure: &FolderStructure, config: &Option<Config>, comfyui_path: Option<PathBuf>) -> Result<()> {
    if let Some(path) = comfyui_path {
        let comfyui_structure: FolderStructure = match config {
            Some(config) => config.clone().comfyui.try_into()?,
            None => ComfyUIConfig::new(path).try_into()?,
        };

        models_structure.soft_link_to(&comfyui_structure)?;
    }

    Ok(())
}

pub fn process_webui(models_structure: &FolderStructure, config: &Option<Config>, webui_path: Option<PathBuf>) -> Result<()> {
    if let Some(path) = webui_path {
        let webui_structure: FolderStructure = match config {
            Some(config) => config.clone().webui.try_into()?,
            None => WebUIConfig::new(path).try_into()?,
        };

        models_structure.soft_link_to(&webui_structure)?;
    }

    Ok(())
}
