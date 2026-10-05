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

/// Index of the closest ancestor of `child` in `ancestors`
pub fn find_ancestor(child: &Path, ancestors: &[PathBuf]) -> usize {
    for ancestor in child.ancestors() {
        if let Some(i) = ancestors.iter().position(|a| a == ancestor) {
            return i;
        }
    }
    panic!("Event detected outside of directories watched?");
}

/// Path given to the command for a changed file, based on the paths as the user typed them:
/// the watched path, or with `recursive`, the changed file under it
pub fn display_path(
    changed: &Path,
    typed: &[PathBuf],
    canon: &[PathBuf],
    recursive: bool,
) -> PathBuf {
    let i = find_ancestor(changed, canon);
    let relative = changed.strip_prefix(&canon[i]).unwrap(); // find_ancestor guarantees it
    if !recursive || relative.as_os_str().is_empty() {
        typed[i].clone()
    } else {
        typed[i].join(relative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let typed = [
            PathBuf::from("src/"),
            PathBuf::from("src/sub"),
            PathBuf::from("Cargo.toml"),
        ];
        let canon = [
            PathBuf::from("/p/src"),
            PathBuf::from("/p/src/sub"),
            PathBuf::from("/p/Cargo.toml"),
        ];
        let show =
            |changed: &str, recursive| display_path(Path::new(changed), &typed, &canon, recursive);

        assert_eq!(show("/p/src/main.rs", false), Path::new("src/"));
        assert_eq!(show("/p/src/main.rs", true), Path::new("src/main.rs"));
        // Closest watched ancestor wins
        assert_eq!(show("/p/src/sub/a.rs", false), Path::new("src/sub"));
        assert_eq!(show("/p/src/sub/a.rs", true), Path::new("src/sub/a.rs"));
        // Watched file itself, no trailing slash added
        assert_eq!(show("/p/Cargo.toml", true), Path::new("Cargo.toml"));
    }
}
