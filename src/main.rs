use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct RelativeFolderStructure {
    checkpoints: String,
    loras: String,
    controlnet: String,
    upscale_models: String,
    vae: String,
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
    fn from_relative(
        value: &RelativeFolderStructure,
        path: &PathBuf,
    ) -> Result<Self, std::io::Error> {
        let absolute_path = path.canonicalize()?;

        let checkpoints = absolute_path.join(value.checkpoints.clone());
        println!("Checkpoints path: {:?}", checkpoints.display());
        let loras = absolute_path.join(value.loras.clone());
        println!("LoRAs path: {:?}", loras.display());
        let controlnet = absolute_path.join(value.controlnet.clone());
        println!("ControlNet path: {:?}", controlnet.display());
        let upscale_models = absolute_path.join(value.upscale_models.clone());
        println!("Upscale models path: {:?}", upscale_models.display());
        let vae = absolute_path.join(value.vae.clone());
        println!("VAE path: {:?}", vae.display());
        Ok(Self {
            checkpoints: checkpoints.canonicalize()?,
            loras: loras.canonicalize()?,
            controlnet: controlnet.canonicalize()?,
            upscale_models: upscale_models.canonicalize()?,
            vae: vae.canonicalize()?,
        })
    }

    fn hard_link_to(&self, to: &Self) -> Result<(), String> {
        println!(
            "Linking from {:?} to {:?}",
            self.checkpoints, to.checkpoints
        );

        println!("Linking from {:?} to {:?}", self.loras, to.loras);

        println!("Linking from {:?} to {:?}", self.controlnet, to.controlnet);

        println!(
            "Linking from {:?} to {:?}",
            self.upscale_models, to.upscale_models
        );

        println!("Linking from {:?} to {:?}", self.vae, to.vae);
        Ok(())
    }
}

impl Default for RelativeFolderStructure {
    fn default() -> Self {
        Self {
            checkpoints: "checkpoints".to_string(),
            loras: "loras".to_string(),
            controlnet: "controlnet".to_string(),
            upscale_models: "upscale_models".to_string(),
            vae: "vae".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
                checkpoints: "Stable-diffusion".to_string(),
                loras: "Lora".to_string(),
                controlnet: "ControlNet".to_string(),
                upscale_models: "ESRGAN".to_string(),
                vae: "VAE".to_string(),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

fn main() -> std::io::Result<()> {
    let config_file_path = "config.toml";
    let config_contents = std::fs::read_to_string(config_file_path).unwrap_or(String::new());
    let config = toml::from_str::<Config>(&config_contents).unwrap();

    let models_path =
        FolderStructure::from_relative(&config.models.config.clone(), &config.models.path)?;

    if config.comfyui.enabled {
        println!("ComfyUI path is set, linking");
        if let Some(ref rel_path) = config.comfyui.path {
            let path = FolderStructure::from_relative(&config.comfyui.config.clone(), rel_path)?;

            let _ = models_path.hard_link_to(&path);
        }
    }

    if config.webui.enabled && config.webui.path.is_some() {
        println!("WebUI path is set");
        if let Some(ref rel_path) = config.webui.path {
            let path = FolderStructure::from_relative(&config.webui.config.clone(), rel_path)?;

            let _ = models_path.hard_link_to(&path);
        }
    }

    #[cfg(debug_assertions)]
    println!("Current config: {:?}", config);
    Ok(())
}
