use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new() -> Self {
        let base = env::temp_dir();
        loop {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before UNIX_EPOCH")
                .as_nanos();
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let candidate = base.join(format!("docs_list_test_{}_{}_{}", process::id(), nanos, id));
            match fs::create_dir(&candidate) {
                Ok(()) => return Self { path: candidate },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("failed to create temp dir: {error}"),
            }
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    fs::write(full_path, contents).expect("failed to write file");
}

fn create_dir(root: &Path, relative_path: &str) {
    fs::create_dir_all(root.join(relative_path)).expect("failed to create directory");
}

fn run_cli(cwd: &Path) -> Output {
    let binary = binary_path();
    Command::new(binary)
        .current_dir(cwd)
        .output()
        .expect("failed to run docs-list CLI")
}

fn run_cli_with_pipefail_head(cwd: &Path) -> Output {
    let binary = binary_path();
    Command::new("bash")
        .arg("-lc")
        .arg("set -o pipefail; \"$BIN\" | head -n 1 > /dev/null")
        .env("BIN", binary)
        .current_dir(cwd)
        .output()
        .expect("failed to run docs-list via pipe")
}

fn binary_path() -> &'static str {
    option_env!("CARGO_BIN_EXE_docs-list")
        .or(option_env!("CARGO_BIN_EXE_docs_list"))
        .expect("expected CARGO_BIN_EXE_docs-list or CARGO_BIN_EXE_docs_list")
}

#[test]
fn exits_with_error_when_docs_directory_is_missing() {
    let temp = TempDirGuard::new();

    let output = run_cli(temp.path());

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "docs:list: missing docs directory. Run from repo root.\n"
    );
}

#[test]
fn exits_with_error_when_docs_path_is_not_directory() {
    let temp = TempDirGuard::new();
    write_file(temp.path(), "docs", "not a directory");

    let output = run_cli(temp.path());

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "docs:list: docs path is not a directory.\n"
    );
}

#[test]
fn lists_markdown_files_and_metadata_with_node_script_parity() {
    let temp = TempDirGuard::new();
    create_dir(temp.path(), "docs");

    write_file(
        temp.path(),
        "docs/a.md",
        "---\nsummary: \"  Alpha   summary \"\nread_when:\n  - first hint\n  - second hint\n---\nbody\n",
    );
    write_file(
        temp.path(),
        "docs/b.md",
        "---\nsummary: Beta summary\n---\nbody\n",
    );
    write_file(
        temp.path(),
        "docs/sub/c.md",
        "---\nsummary: 'Gamma summary'\nread_when: ['  react hooks ', 42, true, null, '', false]\n---\nbody\n",
    );
    write_file(
        temp.path(),
        "docs/bad-frontmatter.md",
        "---\nsummary: Missing closer\nread_when:\n  - ignored\n",
    );
    write_file(
        temp.path(),
        "docs/empty-summary.md",
        "---\nsummary: \"   \"\nread_when: ['still ignored in output']\n---\nbody\n",
    );
    write_file(
        temp.path(),
        "docs/missing-summary.md",
        "---\nread_when:\n  - should not print\n---\nbody\n",
    );
    write_file(temp.path(), "docs/no-frontmatter.md", "plain markdown");
    write_file(
        temp.path(),
        "docs/archive/ignored.md",
        "---\nsummary: archive\n---\n",
    );
    write_file(
        temp.path(),
        "docs/research/ignored.md",
        "---\nsummary: research\n---\n",
    );
    write_file(
        temp.path(),
        "docs/.hidden.md",
        "---\nsummary: hidden file\n---\n",
    );
    write_file(
        temp.path(),
        "docs/.hidden/nested.md",
        "---\nsummary: hidden dir\n---\n",
    );
    write_file(
        temp.path(),
        "docs/sub/.hidden-nested.md",
        "---\nsummary: hidden nested\n---\n",
    );
    write_file(temp.path(), "docs/not-markdown.txt", "skip");

    let output = run_cli(temp.path());

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        concat!(
            "Listing all markdown files in docs folder:\n",
            "a.md - Alpha summary\n",
            "  Read when: first hint; second hint\n",
            "b.md - Beta summary\n",
            "bad-frontmatter.md - [unterminated front matter]\n",
            "empty-summary.md - [summary is empty]\n",
            "missing-summary.md - [summary key missing]\n",
            "no-frontmatter.md - [missing front matter]\n",
            "sub/c.md - Gamma summary\n",
            "  Read when: react hooks; 42; true; false\n",
            "\n",
            "Reminder: keep docs up to date as behavior changes. When your task matches any \"Read when\" hint above (cache directives, database work, tests, etc.), read that doc before coding, and suggest new coverage when it is missing.\n"
        )
    );
}

#[test]
fn handles_epipe_without_nonzero_exit() {
    let temp = TempDirGuard::new();
    create_dir(temp.path(), "docs");
    write_file(temp.path(), "docs/a.md", "---\nsummary: A\n---\n");

    let output = run_cli_with_pipefail_head(temp.path());

    assert_eq!(output.status.code(), Some(0));
}
