use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use blake3::Hash;
use dashmap::DashMap;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub fn get_hashes(paths: Vec<PathBuf>, finished: Arc<AtomicUsize>) -> HashMap<Hash, PathBuf> {
    let items: DashMap<Hash, PathBuf> = DashMap::with_capacity(paths.len());
    paths.par_iter().for_each(|item| {
        let data = std::fs::read(&item).unwrap_or_default();
        let mut hasher = blake3::Hasher::new();
        hasher.update_rayon(&data);
        items.insert(hasher.finalize(), item.clone());
        finished.fetch_add(1, Ordering::Relaxed);
    });
    items.into_iter().collect::<HashMap<Hash, PathBuf>>()
}
