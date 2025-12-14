use std::path::{Path, PathBuf};

pub fn check_path(path: &Path) -> Option<PathBuf> {
    if !path.exists() {
        eprintln!("Path {path:?} does not exist");
        return None;
    }
    if !path.is_dir() && !path.is_file() {
        eprintln!("Path is neither a directory nor a file");
        // What is it then ?
        return None;
    }

    path.canonicalize()
        .inspect_err(|e| {
            eprintln!("Failed to canonicalize path {path:?}");
            eprintln!("Error : {e:?}");
        })
        .ok()
}

pub fn find_ancestor(child: &Path, ancestors: &[PathBuf]) -> PathBuf {
    for ancestor in child.ancestors() {
        let ancestor = ancestor.to_path_buf();
        if ancestors.contains(&ancestor) {
            return ancestor;
        }
    }
    panic!("Event detected outside of directories watched?");
}
