use sha2::Digest;
use std::{collections::HashMap, path::PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum SortError {
    #[error("Could not get EXIF data for {0}")]
    InvalidExifFormat(String),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error walking files: {0}")]
    Walking(#[from] WalkError),
    #[error("EXIF error: {0}")]
    Exif(#[from] exif::Error),
}

pub fn sort(infolder: &PathBuf, outfolder: &std::path::Path) -> Result<(), SortError> {
    let outfolder = outfolder.to_path_buf();
    let pathlist = walk(infolder)?;
    for path in pathlist {
        let mut file_reader = std::io::BufReader::new(std::fs::File::open(&path)?);
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut file_reader)?;
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
    deduplicate_images(outfolder)?;
    Ok(())
}

fn deduplicate_images(path: impl AsRef<std::path::Path>) -> Result<(), WalkError> {
    let paths = walk(path)?;
    let mut items: HashMap<String, PathBuf> = HashMap::with_capacity(paths.len());
    for item in paths {
        if let Some(multipleof) = items.insert(hash_bytes(std::fs::read(&item)?), item.clone()) {
            std::fs::remove_file(multipleof)?;
        }
    }
    Ok(())
}

fn hash_bytes(bytes: Vec<u8>) -> String {
    sha2::Sha512::digest(bytes)
        .into_iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>()
}

#[derive(thiserror::Error, Debug)]
pub enum WalkError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    UnsupportedNodeType(String),
}

fn walk(path: impl AsRef<std::path::Path>) -> Result<Vec<PathBuf>, WalkError> {
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
