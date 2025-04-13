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

impl Default for RelativeFolderStructure {
    fn default() -> Self {
        Self {
            checkpoints: RelativePath::new("checkpoints").to_relative_path_buf(),
            loras: RelativePath::new("loras").to_relative_path_buf(),
            controlnet: RelativePath::new("controlnet").to_relative_path_buf(),
            upscale_models: RelativePath::new("upscale_models").to_relative_path_buf(),
            vae: RelativePath::new("vae").to_relative_path_buf(),
        }
    }
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
    #[serde(default)]
    config: RelativeFolderStructure,
}

impl Default for ComfyUIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            config: Default::default(),
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
    #[serde(default)]
    config: RelativeFolderStructure,
}

impl Default for WebUIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            config: RelativeFolderStructure {
                checkpoints: RelativePath::new("Stable-diffusion").to_relative_path_buf(),
                loras: RelativePath::new("Lora").to_relative_path_buf(),
                controlnet: RelativePath::new("ControlNet").to_relative_path_buf(),
                upscale_models: RelativePath::new("ESRGAN").to_relative_path_buf(),
                vae: RelativePath::new("VAE").to_relative_path_buf(),
            },
        }
    }
}

impl TryInto<FolderStructure> for WebUIConfig {
    type Error = String;

    fn try_into(self) -> Result<FolderStructure, Self::Error> {
        match self.path {
            Some(path) => Ok(FolderStructure::from_relative(path, self.config)),
            None => Err("Path cannot be empty".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct ModelsConfig {
    path: PathBuf,
    #[serde(default)]
    config: RelativeFolderStructure,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            path: Path::new("./models").to_path_buf(),
            config: Default::default(),
        }
    }
}

impl Into<FolderStructure> for ModelsConfig {
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
    models: ModelsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            comfyui: Default::default(),
            webui: Default::default(),
            models: Default::default(),
        }
    }
}

fn main() -> Result<(), String> {
    let config_file_path = "config.toml";
    let config_contents = std::fs::read_to_string(config_file_path).unwrap_or(String::new());
    let config = toml::from_str::<Config>(&config_contents).unwrap();
    println!("Current config: {:?}", config);

    let models_structure: FolderStructure = config.clone().models.into();

    if config.comfyui.enabled {
        println!("ComfyUI path is set, linking");
        let Ok(folder_structure): Result<FolderStructure, String> =
            config.clone().comfyui.try_into()
        else {
            return Err("ComfyUI configuration is invalid".to_string());
        };

        println!("Linking {:#?} to {:#?}", models_structure, folder_structure);
        if let Err(e) = models_structure.soft_link_to(&folder_structure) {
            return Err(format!("Failed to link models: {}", e));
        }
        println!("ComfyUI linked successfully");
    }

    if config.webui.enabled {
        println!("WebUI path is set");
        let Ok(folder_structure): Result<FolderStructure, String> = config.clone().webui.try_into()
        else {
            return Err("WebUI configuration is invalid".to_string());
        };

        println!("Linking {:#?} to {:#?}", models_structure, folder_structure);
        if let Err(e) = models_structure.soft_link_to(&folder_structure) {
            return Err(format!("Failed to link models: {}", e));
        };
        println!("WebUI linked successfully");
    }

    Ok(())
}
