//! No source lives in a directory named `keys`.
//!
//! A directory named `keys` is where key material is kept on disk, and
//! tooling that guards key material treats every file under a directory of
//! that name as secret, whatever the file is. That rule needs no exception
//! for source files only while no source directory uses the name, so the
//! `keys` subcommand's module is `src/key_management/`.

use std::path::{Path, PathBuf};

/// The repository's roots of tracked files. Build output lives outside them.
const SOURCE_ROOTS: &[&str] = &["src", "tests", "docs", ".github"];

/// Collects every directory under `dir` whose name is `keys` in any letter
/// case (such tooling may ignore case).
fn directories_named_keys(dir: &Path, found: &mut Vec<PathBuf>) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        let is_dir = entry
            .file_type()
            .unwrap_or_else(|e| panic!("cannot stat {}: {e}", entry.path().display()))
            .is_dir();
        let name = entry.file_name();
        if !is_dir || name == "target" {
            continue;
        }
        if name.to_string_lossy().eq_ignore_ascii_case("keys") {
            found.push(entry.path());
        } else {
            directories_named_keys(&entry.path(), found);
        }
    }
}

#[test]
fn no_source_directory_is_named_keys() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found = Vec::new();
    for root in SOURCE_ROOTS {
        let root = repo.join(root);
        assert!(
            root.is_dir(),
            "{} is a root of this repository's tracked files",
            root.display()
        );
        directories_named_keys(&root, &mut found);
    }
    assert!(
        found.is_empty(),
        "a directory named `keys` is for key material, never source; rename {found:?}"
    );
}

/// The walk sees a `keys` directory when there is one, in either letter case,
/// and does not descend into build output.
#[test]
fn the_walk_finds_a_keys_directory_and_skips_build_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    for dir in ["a/b/keys", "a/Keys", "target/keys", "a/keyset"] {
        std::fs::create_dir_all(tmp.path().join(dir)).expect("create fixture directory");
    }
    std::fs::write(tmp.path().join("a/keys.rs"), "").expect("create fixture file");

    let mut found = Vec::new();
    directories_named_keys(tmp.path(), &mut found);
    found.sort();
    assert_eq!(
        found,
        [tmp.path().join("a/Keys"), tmp.path().join("a/b/keys")]
    );
}
