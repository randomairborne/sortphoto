use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walker::WalkError;

use crate::hashing::get_hashes;
use crate::walker::walk;

mod hashing;
mod walker;

#[derive(thiserror::Error, Debug, Clone)]
pub enum SortError {
    #[error("Could not get EXIF data for {0}")]
    InvalidExifFormat(String),
    #[error("I/O Error: {0}")]
    Io(Arc<std::io::Error>),
    #[error("Error walking files: {0}")]
    Walking(#[from] WalkError),
    #[error("EXIF error: {0}")]
    Exif(Arc<exif::Error>),
    #[error("Join error: Failed to join task")]
    Join,
}
impl From<std::io::Error> for SortError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Arc::new(e))
    }
}
impl From<exif::Error> for SortError {
    fn from(e: exif::Error) -> Self {
        Self::Exif(Arc::new(e))
    }
}

impl From<Box<dyn std::any::Any + Send>> for SortError {
    fn from(_: Box<dyn std::any::Any + Send>) -> Self {
        Self::Join
    }
}

#[derive(Debug, Clone)]
pub enum SortProgress {
    Started,
    Hashing(f32),
    MovingPhotos(f32),
    Done(String),
    Error(SortError),
}

impl SortProgress {
    pub fn completion(&self) -> f32 {
        match self {
            Self::Started => 0.0,
            Self::Hashing(v) => v * 0.9,
            Self::MovingPhotos(v) => v * 0.1,
            Self::Done(_) => 1.0,
            Self::Error(_) => 0.0,
        }
    }
}

pub fn sort(
    infolder: PathBuf,
    outfolder: PathBuf,
    sender: watch::WatchSender<SortProgress>,
) -> Result<(), SortError> {
    sender.send(SortProgress::Started);
    let mut pathlist = walk(&infolder)?;
    let existing_files = walk(&outfolder)?;
    let in_pathlist = pathlist.clone();
    let total_files = existing_files.len() + pathlist.len();
    let input_file_count = existing_files.len();
    let finished_files = Arc::new(AtomicUsize::new(0));
    let ff1 = finished_files.clone();
    let ff2 = finished_files.clone();
    let inhashes_handle = std::thread::spawn(move || get_hashes(in_pathlist, ff1));
    let outhashes_handle = std::thread::spawn(move || get_hashes(existing_files, ff2));
    while total_files > finished_files.load(Ordering::Relaxed) {
        let percentage = finished_files.load(Ordering::Relaxed) as f32 / total_files as f32;
        sender.send(SortProgress::Hashing(percentage))
    }
    let inhashes = inhashes_handle.join()??;
    let outhashes = outhashes_handle.join()??;
    for hash in outhashes.keys() {
        if let Some(path) = inhashes.get(hash) {
            pathlist.retain(|p| p != path && file_is_image(path.as_path()))
        }
    }
    let deduped_pathlist_length = pathlist.len() as f32;
    let mut moved_counter = 0.0;
    for path in pathlist {
        let mut file_reader = std::io::BufReader::new(std::fs::File::open(&path)?);
        let exifreader = exif::Reader::new();
        let exif = match exifreader.read_from_container(&mut file_reader) {
            Ok(ex) => ex,
            Err(_err) => return Ok(handle_unknown(&path, &outfolder)?),
        };
        if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
            match field.value {
                exif::Value::Ascii(ref vec) if !vec.is_empty() => {
                    if let Ok(datetime) = exif::DateTime::from_ascii(&vec[0]) {
                        let folder_year = datetime.year.to_string();
                        let folder_month = int_to_month_name(datetime.month);
                        let day_raw = datetime.day.to_string();
                        let day_suffix = get_day_suffix(&day_raw);
                        let mut datafolder = outfolder.clone();
                        datafolder.push(folder_year);
                        datafolder.push(folder_month);
                        if let Err(e) = std::fs::create_dir_all(&datafolder) {
                            match e.kind() {
                                std::io::ErrorKind::AlreadyExists => {}
                                _ => {
                                    eprintln!("{e:?}")
                                }
                            }
                        };
                        datafolder.push("dummy_path_will_always_be_overwritten.tmp");
                        let default_name = format!(
                            "{day_raw}{day_suffix}.{}",
                            path.extension()
                                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                                .to_string_lossy()
                        );
                        let mut destination = datafolder.with_file_name(default_name);
                        let mut file_counter = 0;
                        loop {
                            file_counter += 1;
                            if !destination.exists() {
                                break;
                            }
                            destination = datafolder.with_file_name(format!(
                                "{day_raw}{day_suffix} - {file_counter}.{}",
                                path.extension()
                                    .unwrap_or_else(|| std::ffi::OsStr::new(""))
                                    .to_string_lossy()
                            ))
                        }
                        std::fs::copy(&path, &destination)?;
                        moved_counter += 1.0;
                        sender.send(SortProgress::MovingPhotos(
                            moved_counter / deduped_pathlist_length,
                        ))
                    } else {
                        handle_unknown(&path, &outfolder)?;
                    }
                }
                _ => handle_unknown(&path, &outfolder)?,
            }
        } else {
            handle_unknown(&path, &outfolder)?;
        }
    }
    sender.send(SortProgress::Done(format!(
        "Sorting complete! Moved {} files, ignored {} non-image files",
        deduped_pathlist_length,
        input_file_count - deduped_pathlist_length as usize
    )));
    Ok(())
}

fn int_to_month_name(num: u8) -> &'static str {
    match num {
        1 => "1 - January",
        2 => "2 - Febuary",
        3 => "3 - March",
        4 => "4 - April",
        5 => "5 - May",
        6 => "6 - June",
        7 => "7 - July",
        8 => "8 - August",
        9 => "9 - September",
        10 => "10 - October",
        11 => "11 - November",
        12 => "12 - December",
        _ => "Invalid month",
    }
}

fn get_day_suffix(data: &str) -> &'static str {
    match data {
        "1" => "st",
        "2" => "nd",
        "3" => "rd",
        "21" => "st",
        "22" => "nd",
        "23" => "rd",
        "31" => "st",
        "32" => "nd",
        "33" => "rd",
        _ => "th",
    }
}

fn handle_unknown(path: &PathBuf, outfolder: &std::path::Path) -> std::io::Result<()> {
    let mut datafolder = outfolder.to_path_buf();
    datafolder.push("Unknown");
    datafolder.push("dummypath_will_always_be_overwritten.tmp");
    let ext = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_string_lossy();
    let mut file_counter = 0;
    let mut destination = datafolder.with_file_name(format!("0.{ext}"));
    loop {
        file_counter += 1;
        if !destination.exists() {
            break;
        }
        destination = datafolder.with_file_name(format!(
            "{file_counter}.{}",
            path.extension()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_string_lossy()
        ));
    }
    std::fs::copy(&path, &destination)?;
    Ok(())
}

fn file_is_image(path: &std::path::Path) -> bool {
    if let Some(extension_os_str) = path.extension() {
        if let Some(extension) = extension_os_str.to_str() {
            let ext = extension.to_lowercase();
            if matches!(
                ext.as_str(),
                "png" | "tiff" | "jpeg" | "heic" | "heif" | "avif" | "webp"
            ) {
                return true;
            }
        }
    }
    false
}
