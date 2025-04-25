use std::path::Path;
use std::path::PathBuf;

use log::debug;
use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use serde::Deserialize;

use crate::link;

#[derive(Debug, Deserialize, Clone)]
pub struct RelativeFolderStructure {
    pub checkpoints: RelativePathBuf,
    pub loras: RelativePathBuf,
    pub controlnet: RelativePathBuf,
    pub upscale_models: RelativePathBuf,
    pub vae: RelativePathBuf,
    pub embeddings: RelativePathBuf,
}

#[derive(Debug)]
pub struct FolderStructure {
    pub checkpoints: PathBuf,
    pub loras: PathBuf,
    pub controlnet: PathBuf,
    pub upscale_models: PathBuf,
    pub vae: PathBuf,
    pub embeddings: PathBuf,
}

impl FolderStructure {
    pub fn from_relative(base_path: PathBuf, relative_paths: RelativeFolderStructure) -> Self {
        Self {
            checkpoints: relative_paths.checkpoints.to_logical_path(&base_path),
            loras: relative_paths.loras.to_logical_path(&base_path),
            controlnet: relative_paths.controlnet.to_logical_path(&base_path),
            upscale_models: relative_paths.upscale_models.to_logical_path(&base_path),
            vae: relative_paths.vae.to_logical_path(&base_path),
            embeddings: relative_paths.embeddings.to_logical_path(&base_path),
        }
    }

    pub fn hard_link_to(&self, to: &Self) -> Result<(), std::io::Error> {
        let paths = [
            (&self.checkpoints, &to.checkpoints),
            (&self.loras, &to.loras),
            (&self.controlnet, &to.controlnet),
            (&self.upscale_models, &to.upscale_models),
            (&self.vae, &to.vae),
            (&self.embeddings, &to.embeddings),
        ];

        for (from, to_path) in paths {
            debug!("Hard linking {} to {}", from.display(), to_path.display());
            link::create_hard_link(from, to_path)?;
        }

        Ok(())
    }

    pub fn soft_link_to(&self, to: &Self) -> Result<(), std::io::Error> {
        let paths = [
            (&self.checkpoints, &to.checkpoints),
            (&self.loras, &to.loras),
            (&self.controlnet, &to.controlnet),
            (&self.upscale_models, &to.upscale_models),
            (&self.vae, &to.vae),
            (&self.embeddings, &to.embeddings),
        ];

        for (from, to_path) in paths {
            debug!("Soft linking {} to {}", from.display(), to_path.display());
            link::create_symlink(from, to_path)?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComfyUIConfig {
    pub path: PathBuf,
    #[serde(default = "get_default_structure_comfyui")]
    pub config: RelativeFolderStructure,
}

impl ComfyUIConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            config: get_default_structure_comfyui(),
        }
    }
}

pub fn get_default_structure_comfyui() -> RelativeFolderStructure {
    RelativeFolderStructure {
        checkpoints: RelativePath::new("checkpoints").to_relative_path_buf(),
        loras: RelativePath::new("loras").to_relative_path_buf(),
        controlnet: RelativePath::new("controlnet").to_relative_path_buf(),
        upscale_models: RelativePath::new("upscale_models").to_relative_path_buf(),
        vae: RelativePath::new("vae").to_relative_path_buf(),
        embeddings: RelativePath::new("embeddings").to_relative_path_buf(),
    }
}

impl TryFrom<ComfyUIConfig> for FolderStructure {
    type Error = std::io::Error;

    fn try_from(value: ComfyUIConfig) -> Result<Self, Self::Error> {
        Ok(FolderStructure::from_relative(
            value.path,
            value.config.clone(),
        ))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebUIConfig {
    pub path: PathBuf,
    #[serde(default = "get_default_structure_webui")]
    pub config: RelativeFolderStructure,
}

impl WebUIConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            config: get_default_structure_webui(),
        }
    }
}

pub fn get_default_structure_webui() -> RelativeFolderStructure {
    RelativeFolderStructure {
        checkpoints: RelativePath::new("models/Stable-diffusion").to_relative_path_buf(),
        loras: RelativePath::new("models/Lora").to_relative_path_buf(),
        controlnet: RelativePath::new("models/ControlNet").to_relative_path_buf(),
        upscale_models: RelativePath::new("models/ESRGAN").to_relative_path_buf(),
        vae: RelativePath::new("models/VAE").to_relative_path_buf(),
        embeddings: RelativePath::new("embeddings").to_relative_path_buf(),
    }
}

impl TryFrom<WebUIConfig> for FolderStructure {
    type Error = std::io::Error;

    fn try_from(value: WebUIConfig) -> Result<Self, Self::Error> {
        Ok(FolderStructure::from_relative(
            value.path,
            value.config.clone(),
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub comfyui: ComfyUIConfig,
    pub webui: WebUIConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneralConfig {
    pub path: PathBuf,
    #[serde(default = "get_default_structure_general")]
    pub config: RelativeFolderStructure,
}

pub fn get_default_structure_general() -> RelativeFolderStructure {
    RelativeFolderStructure {
        checkpoints: RelativePath::new("checkpoints").to_relative_path_buf(),
        loras: RelativePath::new("loras").to_relative_path_buf(),
        controlnet: RelativePath::new("controlnet").to_relative_path_buf(),
        upscale_models: RelativePath::new("upscale_models").to_relative_path_buf(),
        vae: RelativePath::new("vae").to_relative_path_buf(),
        embeddings: RelativePath::new("embeddings").to_relative_path_buf(),
    }
}

impl GeneralConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            config: get_default_structure_general(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            path: Path::new("./models").to_path_buf(),
            config: get_default_structure_general(),
        }
    }
}

impl From<GeneralConfig> for FolderStructure {
    fn from(value: GeneralConfig) -> Self {
        FolderStructure::from_relative(value.path, value.config)
    }
}
