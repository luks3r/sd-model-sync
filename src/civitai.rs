use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub const API_URL: &str = "https://civitai.com/api/v1/model-versions/by-hash/";

#[derive(Debug)]
pub enum CivitAiError {
    Reqwest(String),
    Unspecified(String),
}

impl std::fmt::Display for CivitAiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CivitAiError::Reqwest(s) => write!(f, "Reqwest: {}", s),
            CivitAiError::Unspecified(s) => write!(f, "Unspecified: {}", s),
        }
    }
}

impl std::error::Error for CivitAiError {}

impl From<&str> for CivitAiError {
    fn from(s: &str) -> Self {
        CivitAiError::Unspecified(s.to_string())
    }
}

impl From<String> for CivitAiError {
    fn from(s: String) -> Self {
        CivitAiError::Unspecified(s)
    }
}

impl From<reqwest::Error> for CivitAiError {
    fn from(e: reqwest::Error) -> Self {
        CivitAiError::Reqwest(e.to_string())
    }
}

type Result<T> = std::result::Result<T, CivitAiError>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ModelInfo {
    pub id: u64,
    #[serde(rename = "modelId")]
    pub model_id: u64,
    pub name: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(rename = "trainedWords")]
    pub trained_words: Vec<Option<String>>,
    #[serde(rename = "trainingStatus")]
    pub training_status: Option<String>,
    #[serde(rename = "trainingDetails")]
    pub training_details: Option<String>,
    #[serde(rename = "baseModel")]
    pub base_model: Option<String>,
    #[serde(rename = "baseModelType")]
    pub base_model_type: Option<String>,
    #[serde(rename = "earlyAccessEndsAt")]
    pub early_access_ends_at: Option<String>,
    #[serde(rename = "earlyAccessConfig")]
    pub early_access_config: Option<EarlyAccessConfig>,
    pub description: Option<String>,
    #[serde(rename = "uploadType")]
    pub upload_type: Option<String>,
    #[serde(rename = "usageControl")]
    pub usage_control: Option<String>,
    pub air: Option<String>,
    pub stats: Stats,
    #[serde(rename = "model")]
    pub model_info: ModelData,
    pub files: Vec<File>,
    pub images: Vec<Image>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EarlyAccessConfig {
    String(String),
    Map(HashMap<String, Value>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "ratingCount")]
    pub rating_count: u64,
    pub rating: f64,
    #[serde(rename = "thumbsUpCount")]
    pub thumbs_up_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelData {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub nsfw: bool,
    pub poi: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ModelType {
    Checkpoint,
    Embedding,
    #[serde(rename = "LORA")]
    Lora,
    Controlnet,
    Upscaler,
    Vae,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelType::Checkpoint => write!(f, "Checkpoint"),
            ModelType::Embedding => write!(f, "Embedding"),
            ModelType::Lora => write!(f, "LoRA"),
            ModelType::Controlnet => write!(f, "Controlnet"),
            ModelType::Upscaler => write!(f, "Upscaler"),
            ModelType::Vae => write!(f, "VAE"),
        }
    }
}

impl ModelType {
    pub fn general_directory(&self) -> &str {
        match self {
            ModelType::Checkpoint => "checkpoints",
            ModelType::Embedding => "embeddings",
            ModelType::Lora => "loras",
            ModelType::Controlnet => "controlnet",
            ModelType::Upscaler => "upscale_models",
            ModelType::Vae => "vae",
        }
    }

    pub fn comfyui_directory(&self) -> &str {
        Self::general_directory(self)
    }

    pub fn webui_directory(&self) -> &str {
        match self {
            ModelType::Checkpoint => "Stable-diffusion",
            ModelType::Embedding => "embeddings",
            ModelType::Lora => "Lora",
            ModelType::Controlnet => "ControlNet",
            ModelType::Upscaler => "ESRGAN",
            ModelType::Vae => "VAE",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub id: u64,
    #[serde(rename = "sizeKB")]
    pub size_kb: f64,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    #[serde(rename = "pickleScanResult")]
    pub pickle_scan_result: Option<String>,
    #[serde(rename = "pickleScanMessage")]
    pub pickle_scan_message: Option<String>,
    #[serde(rename = "virusScanResult")]
    pub virus_scan_result: Option<String>,
    #[serde(rename = "virusScanMessage")]
    pub virus_scan_message: Option<String>,
    #[serde(rename = "scannedAt")]
    pub scanned_at: Option<String>,
    pub metadata: FileMetadata,
    pub hashes: FileHashes,
    pub primary: bool,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub format: Option<String>,
    pub size: Option<String>,
    pub fp: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileHashes {
    #[serde(rename = "AutoV1")]
    pub auto_v1: Option<String>,
    #[serde(rename = "AutoV2")]
    pub auto_v2: Option<String>,
    #[serde(rename = "SHA256")]
    pub sha256: Option<String>,
    #[serde(rename = "CRC32")]
    pub crc32: Option<String>,
    #[serde(rename = "BLAKE3")]
    pub blake3: Option<String>,
    #[serde(rename = "AutoV3")]
    pub auto_v3: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub url: Option<String>,
    #[serde(rename = "nsfwLevel")]
    pub nsfw_level: u32,
    pub width: u32,
    pub height: u32,
    pub hash: Option<String>,
    #[serde(rename = "type")]
    pub image_type: Option<String>,
    pub metadata: ImageMetadata,
    pub meta: Value, // using serde_json::Value to represent an arbitrarily-structured object
    pub availability: Option<String>,
    #[serde(rename = "hasMeta")]
    pub has_meta: bool,
    #[serde(rename = "hasPositivePrompt")]
    pub has_positive_prompt: bool,
    #[serde(rename = "onSite")]
    pub on_site: bool,
    #[serde(rename = "remixOfId")]
    pub remix_of_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageMetadata {
    pub hash: Option<String>,
    pub size: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
impl fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ModelInfo:")?;
        writeln!(f, "  id: {}", self.id)?;
        writeln!(f, "  model_id: {}", self.model_id)?;
        writeln!(f, "  name: {}", self.name.clone().unwrap_or_default())?;
        writeln!(
            f,
            "  created_at: {}",
            self.created_at.clone().unwrap_or_default()
        )?;
        writeln!(
            f,
            "  updated_at: {}",
            self.updated_at.clone().unwrap_or_default()
        )?;
        writeln!(f, "  status: {}", self.status.clone().unwrap_or_default())?;
        writeln!(
            f,
            "  published_at: {}",
            self.published_at.clone().unwrap_or_default()
        )?;
        writeln!(f, "  trained_words: {:?}", self.trained_words)?;
        writeln!(f, "  training_status: {:?}", self.training_status)?;
        writeln!(f, "  training_details: {:?}", self.training_details)?;
        writeln!(
            f,
            "  base_model: {}",
            self.base_model.clone().unwrap_or_default()
        )?;
        writeln!(
            f,
            "  base_model_type: {}",
            self.base_model_type.clone().unwrap_or_default()
        )?;
        writeln!(f, "  early_access_ends_at: {:?}", self.early_access_ends_at)?;
        writeln!(f, "  early_access_config: {:?}", self.early_access_config)?;
        writeln!(f, "  description: {:?}", self.description)?;
        writeln!(
            f,
            "  upload_type: {}",
            self.upload_type.clone().unwrap_or_default()
        )?;
        writeln!(
            f,
            "  usage_control: {}",
            self.usage_control.clone().unwrap_or_default()
        )?;
        writeln!(f, "  air: {}", self.air.clone().unwrap_or_default())?;
        writeln!(f, "  stats: {}", self.stats)?;
        writeln!(f, "  model_info: {}", self.model_info)?;
        writeln!(f, "  files:")?;
        for file in &self.files {
            writeln!(f, "    {}", file)?;
        }
        writeln!(f, "  images:")?;
        for image in &self.images {
            writeln!(f, "    {}", image)?;
        }
        write!(
            f,
            "  download_url: {}",
            self.download_url.clone().unwrap_or_default()
        )
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Downloads: {}, Rating Count: {} (Rating: {}), Thumbs Up: {}",
            self.download_count, self.rating_count, self.rating, self.thumbs_up_count
        )
    }
}

impl fmt::Display for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} (Type: {}, NSFW: {}, POI: {})",
            self.name.clone().unwrap_or_default(),
            self.model_type,
            self.nsfw,
            self.poi
        )
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "File: {} (ID: {})",
            self.name.clone().unwrap_or_default(),
            self.id
        )?;
        writeln!(f, "  Size: {} KB", self.size_kb)?;
        writeln!(f, "  Type: {}", self.file_type.clone().unwrap_or_default())?;
        writeln!(
            f,
            "  Pickle Scan: {} - {:?}",
            self.pickle_scan_result.clone().unwrap_or_default(),
            self.pickle_scan_message
        )?;
        writeln!(
            f,
            "  Virus Scan: {} - {:?}",
            self.virus_scan_result.clone().unwrap_or_default(),
            self.virus_scan_message
        )?;
        writeln!(
            f,
            "  Scanned At: {}",
            self.scanned_at.clone().unwrap_or_default()
        )?;
        writeln!(f, "  Metadata: {}", self.metadata)?;
        writeln!(f, "  Hashes: {}", self.hashes)?;
        writeln!(f, "  Primary: {}", self.primary)?;
        write!(
            f,
            "  Download URL: {}",
            self.download_url.clone().unwrap_or_default()
        )
    }
}

impl fmt::Display for FileMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Format: {}, Size: {}, FP: {}",
            self.format.clone().unwrap_or_default(),
            self.size.clone().unwrap_or_default(),
            self.fp.clone().unwrap_or_default()
        )
    }
}

impl fmt::Display for FileHashes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AutoV1: {}, AutoV2: {}, SHA256: {}, CRC32: {}, BLAKE3: {}, AutoV3: {}",
            self.auto_v1.clone().unwrap_or_default(),
            self.auto_v2.clone().unwrap_or_default(),
            self.sha256.clone().unwrap_or_default(),
            self.crc32.clone().unwrap_or_default(),
            self.blake3.clone().unwrap_or_default(),
            self.auto_v3.clone().unwrap_or_default()
        )
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Image URL: {}", self.url.clone().unwrap_or_default())?;
        writeln!(
            f,
            "  Dimensions: {}x{} (NSFW Level: {})",
            self.width, self.height, self.nsfw_level
        )?;
        writeln!(f, "  Type: {}", self.image_type.clone().unwrap_or_default())?;
        writeln!(f, "  Metadata: {}", self.metadata)?;
        writeln!(f, "  Meta: {}", self.meta)?;
        writeln!(
            f,
            "  Availability: {}",
            self.availability.clone().unwrap_or_default()
        )?;
        writeln!(
            f,
            "  has_meta: {}, has_positive_prompt: {}",
            self.has_meta, self.has_positive_prompt
        )?;
        writeln!(f, "  on_site: {}", self.on_site)?;
        write!(f, "  remix_of_id: {:?}", self.remix_of_id)
    }
}

impl fmt::Display for ImageMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Hash: {}, Size: {} bytes, Dimensions: {}x{}",
            self.hash.clone().unwrap_or_default(),
            self.size.unwrap_or_default(),
            self.width.unwrap_or_default(),
            self.height.unwrap_or_default()
        )
    }
}

pub fn query_model_info(hash: &str) -> Result<ModelInfo> {
    let url = format!("{}{}", API_URL, hash);
    let Ok(resp) = reqwest::blocking::get(url) else {
        return Err("Failed to query Civitai".into());
    };

    if resp.status().is_success() {
        let data: ModelInfo = resp.json()?;
        return Ok(data);
    } else if resp.status().is_server_error() {
        return Err(format!("Civitai Error: {}", resp.status()).into());
    }

    Err("Model not found".into())
}
