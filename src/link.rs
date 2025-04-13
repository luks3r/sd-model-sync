use log::debug;

#[derive(Debug)]
pub enum LinkError {
    Io(String),
    InvalidPath(String),
    Unspecified(String),
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::Io(msg) => f.write_str(msg),
            LinkError::InvalidPath(msg) => f.write_str(msg),
            LinkError::Unspecified(msg) => f.write_str(msg),
        }
    }
}

impl From<std::io::Error> for LinkError {
    fn from(e: std::io::Error) -> Self {
        LinkError::Io(e.to_string())
    }
}

impl From<LinkError> for std::io::Error {
    fn from(e: LinkError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

impl std::error::Error for LinkError {}

type Result<T> = std::result::Result<T, LinkError>;

pub fn create_hard_link(source: &std::path::Path, target: &std::path::Path) -> Result<()> {
    if source.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot hard link directories, use soft_link_to instead",
        )
        .into());
    }

    if target.exists() {
        remove_existing_path(target)?;
    } else {
        ensure_parent_directory(target)?;
    }

    std::fs::hard_link(source, target)?;
    debug!("Created hard link successfully");

    Ok(())
}

pub fn create_symlink(source: &std::path::Path, target: &std::path::Path) -> Result<()> {
    if should_skip_existing_link(source, target)? {
        debug!(
            "Link already exists and points to correct target: {}",
            target.display()
        );
        return Ok(());
    }

    if target.exists() {
        remove_existing_path(target)?;
    }
    ensure_parent_directory(target)?;
    create_platform_specific_symlink(source, target)?;

    debug!("Created symlink successfully");
    Ok(())
}

fn should_skip_existing_link(source: &std::path::Path, target: &std::path::Path) -> Result<bool> {
    if !target.exists() {
        return Ok(false);
    }

    #[cfg(unix)]
    {
        if let Ok(link_target) = std::fs::read_link(target) {
            return Ok(link_target == source);
        }
    }

    #[cfg(windows)]
    {
        return Ok(false);
    }

    Ok(false)
}

fn remove_existing_path(path: &std::path::Path) -> Result<()> {
    debug!("Removing existing path: {}", path.display());

    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
        Ok(())
    } else {
        std::fs::remove_file(path)?;
        Ok(())
    }
}

fn ensure_parent_directory(path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn create_platform_specific_symlink(source: &std::path::Path, target: &std::path::Path) -> Result<()> {
    #[cfg(windows)]
    {
        if source.is_dir() {
            match std::os::windows::fs::symlink_dir(source, target) {
                Ok(_) => Ok(()),
                Err(e) => Err(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create directory symlink. This usually requires administrator \
                        privileges on Windows. Try running the application as administrator. \
                        Original error: {}",
                        e
                    ),
                )
                .into()),
            }
        } else {
            match std::os::windows::fs::symlink_file(source, target) {
                Ok(_) => Ok(()),
                Err(e) => Err(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create file symlink. This may require administrator \
                        privileges on Windows. Original error: {}",
                        e
                    ),
                )
                .into()),
            }
        }
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
}
