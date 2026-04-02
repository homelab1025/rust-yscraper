fn git(args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    let hash = git(&["rev-parse", "HEAD"]);
    let committed_at = git(&["log", "-1", "--format=%cI", "HEAD"]);
    println!("cargo:rustc-env=GIT_HASH={hash}");
    println!("cargo:rustc-env=GIT_COMMITTED_AT={committed_at}");
    println!("cargo:rerun-if-changed=.git/HEAD");
}
