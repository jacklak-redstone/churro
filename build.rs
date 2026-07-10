use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let files: Vec<PathBuf> = WalkDir::new("proto")
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.extension().is_some_and(|ext| ext == "proto"))
        .collect();

    connectrpc_build::Config::new()
        .files(&files)
        .includes(&["proto"])
        .include_file("_connectrpc.rs")
        .compile()
        .unwrap();
}
