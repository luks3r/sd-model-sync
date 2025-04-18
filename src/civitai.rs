use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

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

impl From<reqwest::Error> for CivitAiError {
    fn from(e: reqwest::Error) -> Self {
        CivitAiError::Reqwest(e.to_string())
    }
}

type Result<T> = std::result::Result<T, CivitAiError>;

const API_URL: &str = "https://civitai.com/api/v1/model-versions/by-hash/";

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelInfo {
    pub id: u64,
    #[serde(rename = "modelId")]
    pub model_id: u64,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub status: String,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "trainedWords")]
    pub trained_words: Vec<String>,
    #[serde(rename = "trainingStatus")]
    pub training_status: Option<String>,
    #[serde(rename = "trainingDetails")]
    pub training_details: Option<String>,
    #[serde(rename = "baseModel")]
    pub base_model: String,
    #[serde(rename = "baseModelType")]
    pub base_model_type: String,
    #[serde(rename = "earlyAccessEndsAt")]
    pub early_access_ends_at: Option<String>,
    #[serde(rename = "earlyAccessConfig")]
    pub early_access_config: Option<EarlyAccessConfig>,
    pub description: Option<String>,
    #[serde(rename = "uploadType")]
    pub upload_type: String,
    #[serde(rename = "usageControl")]
    pub usage_control: String,
    pub air: String,
    pub stats: Stats,
    #[serde(rename = "model")]
    pub model_info: ModelData,
    pub files: Vec<File>,
    pub images: Vec<Image>,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
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
    pub rating: u64,
    #[serde(rename = "thumbsUpCount")]
    pub thumbs_up_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelData {
    pub name: String,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub nsfw: bool,
    pub poi: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ModelType {
    Checkpoint,
    Embedding,
    #[serde(rename = "LoRA")]
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
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(rename = "pickleScanResult")]
    pub pickle_scan_result: String,
    #[serde(rename = "pickleScanMessage")]
    pub pickle_scan_message: Option<String>,
    #[serde(rename = "virusScanResult")]
    pub virus_scan_result: String,
    #[serde(rename = "virusScanMessage")]
    pub virus_scan_message: Option<String>,
    #[serde(rename = "scannedAt")]
    pub scanned_at: String,
    pub metadata: FileMetadata,
    pub hashes: FileHashes,
    pub primary: bool,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub format: String,
    pub size: String,
    pub fp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileHashes {
    #[serde(rename = "AutoV1")]
    pub auto_v1: String,
    #[serde(rename = "AutoV2")]
    pub auto_v2: String,
    #[serde(rename = "SHA256")]
    pub sha256: String,
    #[serde(rename = "CRC32")]
    pub crc32: String,
    #[serde(rename = "BLAKE3")]
    pub blake3: String,
    #[serde(rename = "AutoV3")]
    pub auto_v3: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub url: String,
    #[serde(rename = "nsfwLevel")]
    pub nsfw_level: u32,
    pub width: u32,
    pub height: u32,
    pub hash: String,
    #[serde(rename = "type")]
    pub image_type: String,
    pub metadata: ImageMetadata,
    pub meta: Value, // using serde_json::Value to represent an arbitrarily-structured object
    pub availability: String,
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
    pub hash: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
}
impl fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ModelInfo:")?;
        writeln!(f, "  id: {}", self.id)?;
        writeln!(f, "  model_id: {}", self.model_id)?;
        writeln!(f, "  name: {}", self.name)?;
        writeln!(f, "  created_at: {}", self.created_at)?;
        writeln!(f, "  updated_at: {}", self.updated_at)?;
        writeln!(f, "  status: {}", self.status)?;
        writeln!(f, "  published_at: {}", self.published_at)?;
        writeln!(f, "  trained_words: {:?}", self.trained_words)?;
        writeln!(f, "  training_status: {:?}", self.training_status)?;
        writeln!(f, "  training_details: {:?}", self.training_details)?;
        writeln!(f, "  base_model: {}", self.base_model)?;
        writeln!(f, "  base_model_type: {}", self.base_model_type)?;
        writeln!(f, "  early_access_ends_at: {:?}", self.early_access_ends_at)?;
        writeln!(f, "  early_access_config: {:?}", self.early_access_config)?;
        writeln!(f, "  description: {:?}", self.description)?;
        writeln!(f, "  upload_type: {}", self.upload_type)?;
        writeln!(f, "  usage_control: {}", self.usage_control)?;
        writeln!(f, "  air: {}", self.air)?;
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
        write!(f, "  download_url: {}", self.download_url)
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
            self.name, self.model_type, self.nsfw, self.poi
        )
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "File: {} (ID: {})", self.name, self.id)?;
        writeln!(f, "  Size: {} KB", self.size_kb)?;
        writeln!(f, "  Type: {}", self.file_type)?;
        writeln!(
            f,
            "  Pickle Scan: {} - {:?}",
            self.pickle_scan_result, self.pickle_scan_message
        )?;
        writeln!(
            f,
            "  Virus Scan: {} - {:?}",
            self.virus_scan_result, self.virus_scan_message
        )?;
        writeln!(f, "  Scanned At: {}", self.scanned_at)?;
        writeln!(f, "  Metadata: {}", self.metadata)?;
        writeln!(f, "  Hashes: {}", self.hashes)?;
        writeln!(f, "  Primary: {}", self.primary)?;
        write!(f, "  Download URL: {}", self.download_url)
    }
}

impl fmt::Display for FileMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Format: {}, Size: {}, FP: {}",
            self.format, self.size, self.fp
        )
    }
}

impl fmt::Display for FileHashes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AutoV1: {}, AutoV2: {}, SHA256: {}, CRC32: {}, BLAKE3: {}, AutoV3: {}",
            self.auto_v1, self.auto_v2, self.sha256, self.crc32, self.blake3, self.auto_v3
        )
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Image URL: {}", self.url)?;
        writeln!(
            f,
            "  Dimensions: {}x{} (NSFW Level: {})",
            self.width, self.height, self.nsfw_level
        )?;
        writeln!(f, "  Type: {}", self.image_type)?;
        writeln!(f, "  Metadata: {}", self.metadata)?;
        writeln!(f, "  Meta: {}", self.meta)?;
        writeln!(f, "  Availability: {}", self.availability)?;
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
            self.hash, self.size, self.width, self.height
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
    }

    Err("Model not found".into())
}
