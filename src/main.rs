use std::path::{Path, PathBuf};

use relative_path::{RelativePath, RelativePathBuf};
use serde::Deserialize;

mod link;

#[derive(Debug, Deserialize, Clone)]
struct RelativeFolderStructure {
    checkpoints: RelativePathBuf,
    loras: RelativePathBuf,
    controlnet: RelativePathBuf,
    upscale_models: RelativePathBuf,
    vae: RelativePathBuf,
}

#[derive(Debug)]
struct FolderStructure {
    checkpoints: PathBuf,
    loras: PathBuf,
    controlnet: PathBuf,
    upscale_models: PathBuf,
    vae: PathBuf,
}

impl FolderStructure {
    fn from_relative(base_path: PathBuf, relative_paths: RelativeFolderStructure) -> Self {
        Self {
            checkpoints: relative_paths.checkpoints.to_logical_path(&base_path),
            loras: relative_paths.loras.to_logical_path(&base_path),
            controlnet: relative_paths.controlnet.to_logical_path(&base_path),
            upscale_models: relative_paths.upscale_models.to_logical_path(&base_path),
            vae: relative_paths.vae.to_logical_path(&base_path),
        }
    }

    fn hard_link_to(&self, to: &Self) -> Result<(), std::io::Error> {
        let paths = [
            (&self.checkpoints, &to.checkpoints),
            (&self.loras, &to.loras),
            (&self.controlnet, &to.controlnet),
            (&self.upscale_models, &to.upscale_models),
            (&self.vae, &to.vae),
        ];

        for (from, to_path) in paths {
            println!("Hard linking {} to {}", from.display(), to_path.display());
            link::create_hard_link(&from, &to_path)?;
        }

        Ok(())
    }

    fn soft_link_to(&self, to: &Self) -> Result<(), std::io::Error> {
        let paths = [
            (&self.checkpoints, &to.checkpoints),
            (&self.loras, &to.loras),
            (&self.controlnet, &to.controlnet),
            (&self.upscale_models, &to.upscale_models),
            (&self.vae, &to.vae),
        ];

        for (from, to_path) in paths {
            println!("Soft linking {} to {}", from.display(), to_path.display());
            link::create_symlink(&from, &to_path)?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
struct ComfyUIConfig {
    enabled: bool,
    path: Option<PathBuf>,
    #[serde(default = "get_default_structure_comfyui")]
    config: RelativeFolderStructure,
}

fn get_default_structure_comfyui() -> RelativeFolderStructure {
    RelativeFolderStructure {
        checkpoints: RelativePath::new("checkpoints").to_relative_path_buf(),
        loras: RelativePath::new("loras").to_relative_path_buf(),
        controlnet: RelativePath::new("controlnet").to_relative_path_buf(),
        upscale_models: RelativePath::new("upscale_models").to_relative_path_buf(),
        vae: RelativePath::new("vae").to_relative_path_buf(),
    }
}

impl Default for ComfyUIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            config: get_default_structure_comfyui(),
        }
    }
}

impl TryInto<FolderStructure> for ComfyUIConfig {
    type Error = String;

    fn try_into(self) -> Result<FolderStructure, Self::Error> {
        match self.path {
            Some(path) => Ok(FolderStructure::from_relative(path, self.config)),
            None => Err("Path cannot be empty".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct WebUIConfig {
    enabled: bool,
    path: Option<PathBuf>,
    #[serde(default = "get_default_structure_webui")]
    config: RelativeFolderStructure,
}

fn get_default_structure_webui() -> RelativeFolderStructure {
    RelativeFolderStructure {
        checkpoints: RelativePath::new("Stable-diffusion").to_relative_path_buf(),
        loras: RelativePath::new("Lora").to_relative_path_buf(),
        controlnet: RelativePath::new("ControlNet").to_relative_path_buf(),
        upscale_models: RelativePath::new("ESRGAN").to_relative_path_buf(),
        vae: RelativePath::new("VAE").to_relative_path_buf(),
    }
}

impl Default for WebUIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            config: get_default_structure_webui(),
        }
    }
}

impl TryInto<FolderStructure> for WebUIConfig {
    type Error = String;

    fn try_into(self) -> Result<FolderStructure, Self::Error> {
        match self.path {
            Some(path) => Ok(FolderStructure::from_relative(path, self.config.clone())),
            None => Err("Path cannot be empty".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct GeneralConfig {
    path: PathBuf,
    #[serde(default = "get_default_structure_general")]
    config: RelativeFolderStructure,
}

fn get_default_structure_general() -> RelativeFolderStructure {
    RelativeFolderStructure {
        checkpoints: RelativePath::new("checkpoints").to_relative_path_buf(),
        loras: RelativePath::new("loras").to_relative_path_buf(),
        controlnet: RelativePath::new("controlnet").to_relative_path_buf(),
        upscale_models: RelativePath::new("upscale_models").to_relative_path_buf(),
        vae: RelativePath::new("vae").to_relative_path_buf(),
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

impl Into<FolderStructure> for GeneralConfig {
    fn into(self) -> FolderStructure {
        FolderStructure::from_relative(self.path.clone(), self.config.clone())
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    #[serde(default)]
    comfyui: ComfyUIConfig,
    #[serde(default)]
    webui: WebUIConfig,
    #[serde(default)]
    general: GeneralConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            comfyui: Default::default(),
            webui: Default::default(),
            general: Default::default(),
        }
    }
}

fn main() -> Result<(), String> {
    let config_file_path = "config.toml";
    let config_contents = std::fs::read_to_string(config_file_path).unwrap_or(String::new());
    let config = toml::from_str::<Config>(&config_contents).unwrap();
    println!("Current config: {:?}", config);

    let models_structure: FolderStructure = config.clone().general.into();

    if config.comfyui.enabled {
        println!("ComfyUI path is set, linking");
        let Ok(comfyui_config): Result<FolderStructure, String> = config.clone().comfyui.try_into()
        else {
            return Err("ComfyUI configuration is invalid".to_string());
        };

        println!("Linking {:#?} to {:#?}", models_structure, comfyui_config);
        if let Err(e) = models_structure.soft_link_to(&comfyui_config) {
            return Err(format!("Failed to link models: {}", e));
        }
        println!("ComfyUI linked successfully");
    }

    if config.webui.enabled {
        println!("WebUI path is set");
        let Ok(webui_config): Result<FolderStructure, String> = config.clone().webui.try_into()
        else {
            return Err("WebUI configuration is invalid".to_string());
        };

        println!("Linking {:#?} to {:#?}", models_structure, webui_config);
        if let Err(e) = models_structure.soft_link_to(&webui_config) {
            return Err(format!("Failed to link models: {}", e));
        };
        println!("WebUI linked successfully");
    }

    Ok(())
}
