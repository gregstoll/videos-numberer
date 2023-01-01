use regex::Regex;
use std::{ffi::OsStr, path::{Path, PathBuf}, collections::HashMap};
use lazy_static::lazy_static;

use walkdir::WalkDir;

const VIDEO_EXTENSION: &str = "mkv";

fn main() {
    println!("Hello, world!");
}

/// Maps a Vec of paths into their new filenames
fn map_video_paths(paths: &Vec<PathBuf>) -> HashMap<&PathBuf, String> {
    if paths.len() >= 1000 {
        panic!("too many videos!");
    }
    let mut sorted_paths = paths.iter().map(|p| (p, get_sortable_filename(p))).collect::<Vec<_>>();
    // this .clone() is ugly :-(
    sorted_paths.sort_by_key(|p| p.1.clone());
    let digits = if sorted_paths.len() >= 100 { 3 } else { 2 };

    let mut name_mapping = HashMap::new();
    for (index, (original_path, filename)) in sorted_paths.into_iter().enumerate() {
        let new_filename = format!("{:0width$}_{}", (index + 1), filename, width = digits);
        assert!(name_mapping.insert(original_path, new_filename).is_none());
    }
    name_mapping
}

// TODO - take in directory
fn get_video_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    let video_extension = OsStr::new(VIDEO_EXTENSION);
    for e in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() && e.path().extension() == Some(video_extension) {
            paths.push(e.path().to_path_buf());
        }
    }
    paths
}

fn get_sortable_filename(path: &Path) -> String {
    let f = path.file_name().unwrap().to_str().unwrap();
    lazy_static! {
        static ref LEADING_NUMBERS_RE: Regex = Regex::new(r"^\d{1,3}_(.*)$").unwrap();
    }
    if let Some(captures) = LEADING_NUMBERS_RE.captures(f) {
        return captures[1].to_string().to_lowercase();
    }
    return f.to_string().to_lowercase();
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::map_video_paths;

    #[test]
    fn one_entry() {
        let input = vec![PathBuf::from("/abc.mkv")];
        let map = map_video_paths(&input);
        assert_eq!(1, map.len());
        assert_eq!("01_abc.mkv", map[&input[0]]);
    }
}