use std::io::BufReader;
use std::path::Path;

use data_encoding::HEXUPPER;
use ring::digest::SHA256;

#[derive(Debug)]
pub enum EldenError {
    Io(String),
    Hash(String),
}

impl std::fmt::Display for EldenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EldenError::Io(s) => write!(f, "{}", s),
            EldenError::Hash(s) => write!(f, "{}", s),
        }
    }
}

impl From<std::io::Error> for EldenError {
    fn from(err: std::io::Error) -> Self {
        EldenError::Io(err.to_string())
    }
}

impl From<ring::error::Unspecified> for EldenError {
    fn from(err: ring::error::Unspecified) -> Self {
        EldenError::Hash(err.to_string())
    }
}

impl std::error::Error for EldenError {}

type Result<T> = std::result::Result<T, EldenError>;

pub struct EldenRing;
impl EldenRing {
    pub fn calculate_hash_sha256<R: std::io::Read>(mut reader: R) -> Result<String> {
        let mut context = ring::digest::Context::new(&SHA256);
        let mut buffer = [0; 1024 * 64];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }

        let digest = context.finish();
        let hash = HEXUPPER.encode(digest.as_ref());

        Ok(hash)
    }

    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Result<String> {
        let reader = BufReader::new(std::fs::File::open(filepath)?);
        Self::calculate_hash_sha256(reader)
    }
}
