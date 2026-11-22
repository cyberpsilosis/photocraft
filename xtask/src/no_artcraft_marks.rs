//! Guard for this fork (cyberpsilosis/photocraft): the ArtCraft logo and mark files are removed, as
//! `docs/brand/LICENSE-brand.txt` requires of a modified version (Elijah, 2026-11-21: "Take the
//! logos out"). The test fails if one comes back, as a file or as a reference in a text file.
//! It runs with `cargo test --workspace`, because GitHub runs no CI on this fork.

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    /// The removed files' names began with the two strings below; they are built from parts so this
    /// file does not match itself.
    fn needles() -> [String; 2] {
        [["artcraft", "-logo"].concat(), ["artcraft", "-mark"].concat()]
    }

    /// Every tracked file plus every untracked one that is not ignored, so a file about to be
    /// committed is caught too (build output, `corpus/` and other ignored folders are skipped).
    /// A source tree without `.git` (a GitHub zip download) has no `git ls-files`, so the guard
    /// walks the tree instead and skips the folders `.gitignore` keeps out.
    fn repo_files() -> Vec<String> {
        let root = crate::root();
        if !root.join(".git").exists() {
            let mut files = Vec::new();
            walk(&root, &root, &mut files);
            return files;
        }
        let out = Command::new("git")
            .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
            .current_dir(&root)
            .output()
            .expect("git runs in a git checkout");
        assert!(out.status.success(), "git ls-files failed: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).split('\0').filter(|p| !p.is_empty()).map(str::to_owned).collect()
    }

    /// The folders `.gitignore` keeps out (build output, fetched corpora and fonts, local notes),
    /// which the walk without git skips. No tracked folder has one of these names.
    const SKIP: [&str; 10] = [".git", "target", "dist", "corpus", "log", "secrets", "temp", "plan", "craft-fonts", "output"];

    fn walk(root: &Path, dir: &Path, files: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !SKIP.contains(&name.as_str()) {
                    walk(root, &path, files);
                }
            } else if let Ok(rel) = path.strip_prefix(root) {
                files.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    #[test]
    fn no_artcraft_mark_files_or_references() {
        let root = crate::root();
        let needles = needles();
        let mut hits = Vec::new();
        for path in repo_files() {
            let lower = path.to_lowercase();
            if needles.iter().any(|n| lower.contains(n.as_str())) {
                hits.push(format!("{path}: a mark file"));
            }
            // A tracked file deleted from the working tree, or a binary one, has no text to check.
            let Ok(bytes) = std::fs::read(root.join(&path)) else { continue };
            let Ok(text) = std::str::from_utf8(&bytes) else { continue };
            for (i, line) in text.lines().enumerate() {
                let line = line.to_lowercase();
                if needles.iter().any(|n| line.contains(n.as_str())) {
                    hits.push(format!("{path}:{}: names a mark file", i + 1));
                }
            }
        }
        assert!(
            hits.is_empty(),
            "the ArtCraft logos and marks were removed from this fork and must stay out \
             (docs/brand/LICENSE-brand.txt, \"Forks and modified versions\"):\n{}",
            hits.join("\n")
        );
    }

    #[test]
    fn the_guard_sees_a_mark_reference() {
        let needles = needles();
        let sample = ["<img alt=\"x\" src=\"docs/brand/ARTCRAFT", "-MARK.svg\">"].concat().to_lowercase();
        assert!(needles.iter().any(|n| sample.contains(n.as_str())));
        assert!(!needles.iter().any(|n| "PhotoCraft by the ArtCraft team".to_lowercase().contains(n.as_str())));
    }

    #[test]
    fn the_walk_without_git_finds_a_mark_file_and_skips_ignored_folders() {
        let tmp = std::env::temp_dir().join(format!("xtask-mark-guard-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let mark = ["artcraft", "-mark.svg"].concat();
        std::fs::create_dir_all(tmp.join("docs/brand")).expect("make docs/brand");
        std::fs::create_dir_all(tmp.join("target/debug")).expect("make target");
        std::fs::write(tmp.join("docs/brand").join(&mark), "<svg/>").expect("write mark");
        std::fs::write(tmp.join("target/debug").join(&mark), "<svg/>").expect("write build copy");
        let mut files = Vec::new();
        walk(&tmp, &tmp, &mut files);
        let _ = std::fs::remove_dir_all(&tmp);
        assert_eq!(files, vec![format!("docs/brand/{mark}")]);
    }
}
