use std::{path::PathBuf, str::FromStr};

fn main() {
    println!("Hello! This is SortPhoto, an easy tool to take your photos and categorize them by EXIF date!\nThis tool was made by valkyrie_pilot. For support, please visit https://github.com/randomairborne/sortphoto/issues and make an issue.");
    let infolder_text = std::env::args().collect::<Vec<String>>().get(1).on_err("This program takes two arguments: The folder containing the photos you want organized, and the folder you want them put into! You're missing the first one.").clone();
    let outfolder_text = std::env::args().collect::<Vec<String>>().get(2).on_err("This program takes two arguments: The folder containing the photos you want organized, and the folder you want them put into! You're missing the second one.").clone();
    let infolder =
        PathBuf::from_str(&infolder_text).on_err("The first argument is not a valid path!");
    let outfolder =
        PathBuf::from_str(&outfolder_text).on_err("The second argument is not a valid path!");
    if !infolder.exists() {
        std::fs::create_dir_all(&infolder).on_err("Failed to create input folder!");
    }
    if !outfolder.exists() {
        std::fs::create_dir_all(&outfolder).on_err("Failed to create output folder!");
    }
    if !infolder.is_dir() {
        Err("").on_err("The input folder must be a folder!")
    }
    if !outfolder.is_dir() {
        Err("").on_err("The output folder must be a folder!")
    }
    let pathlist = walk(infolder);
    for path in pathlist {
        let file =
            std::fs::File::open(&path).on_err(format!("Failed to open file {}", path.display()));
        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = exif::Reader::new();
        let exif = match exifreader.read_from_container(&mut bufreader) {
            Ok(val) => val,
            Err(_) => {
                eprintln!(
                    "\u{001b}[33;1mFailed to get EXIF metadata for {}\u{001b}[0m",
                    path.display()
                );
                continue;
            }
        };
        if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
            match field.value {
                exif::Value::Ascii(ref vec) if !vec.is_empty() => {
                    if let Ok(datetime) = exif::DateTime::from_ascii(&vec[0]) {
                        let folder_year = datetime.year.to_string();
                        let folder_month = int_to_month_name(datetime.month);
                        let day_raw = datetime.day.to_string();
                        let day_suffix = match day_raw.as_str() {
                            "1" => "st",
                            "2" => "nd",
                            "3" => "rd",
                            _ => "th",
                        };
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
                        datafolder.push("dummypath_will_always_be_overwritten.tmp");
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
                        std::fs::copy(&path, &destination).on_err(format!(
                            "Failed to copy {} to {}",
                            path.display(),
                            destination.display()
                        ));
                    } else {
                        warn(format!(
                            "Unable to extract date and time from EXIF metadata for file {}",
                            path.display()
                        ))
                    }
                }
                _ => warn(format!(
                    "EXIF metadata was in the wrong format for file {}",
                    path.display()
                )),
            }
        } else {
            warn(format!(
                "Failed to get EXIF date and time metadata for file {}",
                path.display()
            ))
        }
    }
    println!("Photos sorted!");
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

fn walk(path: impl AsRef<std::path::Path>) -> Vec<PathBuf> {
    let mut pathlist: Vec<PathBuf> = Vec::new();
    let items = path
        .as_ref()
        .read_dir()
        .on_err("Unable to list the folder contents!");
    for maybe_item in items {
        if let Ok(item) = maybe_item {
            let kind = item
                .file_type()
                .on_err(format!("Unable to read filetype of {item:?}"));
            if kind.is_dir() {
                let mut files = walk(item.path());
                pathlist.append(&mut files);
            } else if kind.is_file() {
                pathlist.push(item.path())
            } else {
                eprintln!(
                    "\u{001b}[33;1mUnable to handle {}\u{001b}[0m",
                    item.path().to_string_lossy()
                );
            }
        } else {
            eprintln!("\u{001b}[33;1mUnable to handle {maybe_item:?}\u{001b}[0m");
        }
    }
    pathlist
}

fn warn(text: impl AsRef<str>) {
    eprintln!("\u{001b}[33;1m{}\u{001b}[0m", text.as_ref());
}

trait OnError<T> {
    fn on_err(self, err: impl AsRef<str>) -> T;
}

impl<T, E> OnError<T> for Result<T, E> {
    fn on_err(self, err: impl AsRef<str>) -> T {
        if let Ok(val) = self {
            val
        } else {
            eprintln!("\n\u{001b}[31;1mOops!\u{001b}[0m {}", err.as_ref());
            std::process::exit(1);
        }
    }
}

impl<T> OnError<T> for Option<T> {
    fn on_err(self, err: impl AsRef<str>) -> T {
        if let Some(val) = self {
            val
        } else {
            eprintln!("\n\u{001b}[31;1mOops!\u{001b}[0m {}", err.as_ref());
            std::process::exit(1);
        }
    }
}
