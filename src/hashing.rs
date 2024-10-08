use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use blake3::Hash;

pub fn get_hashes(
    paths: Vec<PathBuf>,
    finished: Arc<AtomicUsize>,
) -> Result<HashMap<Hash, PathBuf>, std::io::Error> {
    let mut items: HashMap<Hash, PathBuf> = HashMap::with_capacity(paths.len());
    for item in paths {
        let data = std::fs::read(&item)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update_rayon(&data);
        items.insert(hasher.finalize(), item.clone());
        finished.fetch_add(1, Ordering::Relaxed);
    }
    Ok(items)
}
