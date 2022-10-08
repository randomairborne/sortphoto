use std::{path::PathBuf, sync::Arc};

#[derive(thiserror::Error, Debug, Clone)]
pub enum WalkError {
    #[error("{0}")]
    Io(Arc<std::io::Error>),
    #[error("{0}")]
    UnsupportedNodeType(String),
}

impl From<std::io::Error> for WalkError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Arc::new(e))
    }
}

pub fn walk(path: impl AsRef<std::path::Path>) -> Result<Vec<PathBuf>, WalkError> {
    let mut pathlist: Vec<PathBuf> = Vec::new();
    let items = path.as_ref().read_dir()?;
    for item in items {
        let item = item?;
        let kind = item.file_type()?;
        if kind.is_dir() {
            let mut files = walk(item.path())?;
            pathlist.append(&mut files);
        } else if kind.is_file() {
            pathlist.push(item.path())
        } else {
            return Err(WalkError::UnsupportedNodeType(format!(
                "Unable to handle {}",
                item.path().to_string_lossy()
            )));
        }
    }
    Ok(pathlist)
}
