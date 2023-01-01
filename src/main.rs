use regex::Regex;
use std::{ffi::OsStr, path::{Path, PathBuf}, collections::HashMap};
use lazy_static::lazy_static;

use walkdir::WalkDir;

const VIDEO_EXTENSION: &str = "mkv";

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Requires exactly 1 argument - the name of the directory to traverse");
    }
    let paths = get_video_paths(Path::new(&args[1]));
    let map = map_video_paths(&paths);
    for path in &paths {
        let new_filename = &map[path];
        let new_path = path.with_file_name(new_filename);
        println!("Renaming {} to {}", path.display(), new_path.display());
        std::fs::rename(path, new_path).expect("failed to rename file!");
    }
}

/// Maps a Vec of paths into their new filenames
fn map_video_paths(paths: &Vec<PathBuf>) -> HashMap<&PathBuf, String> {
    if paths.len() >= 1000 {
        panic!("too many videos!");
    }
    let mut sorted_paths = paths.iter().map(|p| (p, get_raw_filename(p), get_raw_filename(p).to_lowercase())).collect::<Vec<_>>();
    // this .clone() is ugly :-(
    sorted_paths.sort_by_key(|p| p.2.clone());
    let digits = if sorted_paths.len() >= 100 { 3 } else { 2 };

    let mut name_mapping = HashMap::new();
    for (index, (original_path, filename, _)) in sorted_paths.into_iter().enumerate() {
        let new_filename = format!("{:0width$}_{}", (index + 1), filename, width = digits);
        assert!(name_mapping.insert(original_path, new_filename).is_none());
    }
    name_mapping
}

fn get_video_paths(path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![];
    let video_extension = OsStr::new(VIDEO_EXTENSION);
    for e in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() && e.path().extension() == Some(video_extension) {
            paths.push(e.path().to_path_buf());
        }
    }
    paths
}

fn get_raw_filename(path: &Path) -> String {
    let f = path.file_name().unwrap().to_str().unwrap();
    lazy_static! {
        static ref LEADING_NUMBERS_RE: Regex = Regex::new(r"^\d{1,3}_(.*)$").unwrap();
    }
    if let Some(captures) = LEADING_NUMBERS_RE.captures(f) {
        return captures[1].to_string()
    }
    f.to_string()
}


#[cfg(test)]
mod tests {
    use std::{path::PathBuf};

    use crate::map_video_paths;

    fn get_input_from_strs(strs: &Vec<&str>) -> Vec<PathBuf> {
        strs.iter().map(|s| PathBuf::from(s)).collect::<Vec<_>>()
    }

    fn map_and_assert_values(strs: &Vec<&str>, expected_values: &Vec<&str>) {
        assert_eq!(strs.len(), expected_values.len());
        let input = get_input_from_strs(strs);
        let map = map_video_paths(&input);
        assert_eq!(strs.len(), map.len());
        for (index, &expected) in expected_values.iter().enumerate() {
            assert_eq!(expected, map[&input[index]]);
        }
    }

    #[test]
    fn one_entry() {
        let strs = vec!["/abc.mkv"];
        map_and_assert_values(&strs, &vec!["01_abc.mkv"]);
    }

    #[test]
    fn multiple_entries_split() {
        let strs = vec!["/Movies/a.mkv", "/Movies/c.mkv", "/TV Shows/b.mkv"];
        let expected = vec!["01_a.mkv", "03_c.mkv", "02_b.mkv"];
        map_and_assert_values(&strs, &expected);
    }

    #[test]
    fn multiple_entries_split_preserve_case() {
        let strs = vec!["/Movies/A.mkv", "/Movies/c.mkv", "/TV Shows/B.mkv"];
        let expected = vec!["01_A.mkv", "03_c.mkv", "02_B.mkv"];
        map_and_assert_values(&strs, &expected);
    }

    #[test]
    fn even_more_nested_entries() {
        let strs = vec!["/Movies/a.mkv", "/Movies/c.mkv", "/TV Shows/B/b_1.mkv", "/TV Shows/B/b_2.mkv"];
        let expected = vec!["01_a.mkv", "04_c.mkv", "02_b_1.mkv", "03_b_2.mkv"];
        map_and_assert_values(&strs, &expected);
    }

    #[test]
    fn even_more_nested_entries_after_running() {
        let strs = vec!["/Movies/01_a.mkv", "/Movies/02_c.mkv", "/TV Shows/B/03_b_1.mkv", "/TV Shows/B/04_b_2.mkv"];
        let expected = vec!["01_a.mkv", "04_c.mkv", "02_b_1.mkv", "03_b_2.mkv"];
        map_and_assert_values(&strs, &expected);
    }

}