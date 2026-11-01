use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
fn collect(root: &Path, out: &mut Vec<PathBuf>) {
    // Directory tracking also invalidates this identity when a member is added
    // or removed; tracking only the previous leaf set misses additions.
    println!("cargo:rerun-if-changed={}", root.display());
    let mut entries: Vec<_> = fs::read_dir(root)
        .expect("build identity input directory")
        .map(|x| x.expect("build identity directory entry").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}
fn core_graph(tree: &str) -> Option<String> {
    // Cargo prints a blank separator between each selected package tree.
    // Dependency rows and their actual feature lists remain exact.
    let lines: Vec<_> = tree
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let mut graphs = std::collections::BTreeSet::new();
    for (index, line) in lines.iter().enumerate() {
        let prefix = line.bytes().take_while(u8::is_ascii_digit).count();
        let level: usize = line.get(..prefix)?.parse().ok()?;
        if !line.get(prefix..)?.starts_with("game_core v") {
            continue;
        }
        let mut graph = Vec::new();
        for child in &lines[index + 1..] {
            let prefix = child.bytes().take_while(u8::is_ascii_digit).count();
            let depth: usize = child.get(..prefix)?.parse().ok()?;
            if depth <= level {
                break;
            }
            graph.push(format!("{}{}", depth - level, child.get(prefix..)?));
        }
        graphs.insert(graph.join("\n"));
    }
    (graphs.len() == 1)
        .then(|| graphs.into_iter().next())
        .flatten()
}
fn prepared_graph(
    workspace: &Path,
    compiler: &str,
    target: &str,
    profile: &str,
    features: &str,
) -> Option<String> {
    println!("cargo:rerun-if-env-changed=CMVA_PREPARED_BUILD");
    let path = env::var_os("CMVA_PREPARED_BUILD")?;
    println!("cargo:rerun-if-changed={}", PathBuf::from(&path).display());
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    if value["schema"] != "cmva.prepared-build/v1"
        || value["workspace_root"] != workspace.to_str()?
        || value["compiler"] != compiler
        || value["target"] != target
        || value["profile"] != profile
        || profile != "release"
        || value["core_features"] != features
        || value["packages"]
            != serde_json::json!([
                "game_core",
                "game_server",
                "u11_independent_public_probe",
                "u12_independent_lifecycle_probe"
            ])
    {
        return None;
    }
    for file in ["Cargo.toml", "Cargo.lock", "core_enhanced/Cargo.toml", "game_server/Cargo.toml", "core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/probe/Cargo.toml", "core_enhanced/docs/goals/core_expedition_lifecycle/execution/single_goal/test_parts/runtime001/probe/Cargo.toml"] {
        let bytes = fs::read(workspace.join(file)).ok()?;
        if value["files"][file] != format!("{:x}", Sha256::digest(bytes)) { return None; }
    }
    for name in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTUP_TOOLCHAIN",
    ] {
        let actual = env::var(name).unwrap_or_default();
        // Cargo sets RUSTC for build scripts; the tool records the effective
        // compiler separately, so an unset caller RUSTC is equivalent here.
        if name == "RUSTC" && value["environment"][name] == "" {
            continue;
        }
        if value["environment"][name] != actual {
            return None;
        }
    }
    let tree_path = value["tree_path"].as_str()?;
    println!("cargo:rerun-if-changed={tree_path}");
    let tree = fs::read_to_string(tree_path).ok()?;
    if value["tree_sha256"] != format!("{:x}", Sha256::digest(tree.as_bytes())) {
        return None;
    }
    let graph = core_graph(&tree)?;
    let graph = format!("{:x}", Sha256::digest(graph.as_bytes()));
    (value["core_dependency_graph_sha256"] == graph).then_some(graph)
}
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let workspace = root.parent().unwrap();
    let mut files = Vec::new();
    collect(&root.join("src"), &mut files);
    collect(&root.join("../game_resources/data"), &mut files);
    collect(&root.join("../game_server/src"), &mut files);
    files.extend([
        root.join("Cargo.toml"),
        root.join("build.rs"),
        root.join("../game_server/Cargo.toml"),
        root.join("../Cargo.toml"),
        root.join("../Cargo.lock"),
        root.join("../config/runtime_safety.ron"),
    ]);
    for directory in [
        workspace.join(".cargo"),
        root.join(".cargo"),
        workspace.join("game_server/.cargo"),
    ] {
        println!("cargo:rerun-if-changed={}", directory.display());
        if directory.is_dir() {
            collect(&directory, &mut files);
        }
    }
    // Cargo's host-selected config is an effective build input even though it
    // is outside the source tree. Only config bytes are hashed; no credential
    // file or contents are emitted.
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".cargo")));
    let mut host_configs = Vec::new();
    if let Some(home) = cargo_home {
        for name in ["config", "config.toml"] {
            let file = home.join(name);
            println!("cargo:rerun-if-changed={}", file.display());
            if file.is_file() {
                host_configs.push((name, fs::read(file).expect("effective Cargo config")));
            }
        }
    }
    files.sort();
    files.dedup();
    let mut digest = Sha256::new();
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        let relative = file
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| {
                format!(
                    "workspace/{}",
                    file.strip_prefix(root.parent().unwrap()).unwrap().display()
                )
            });
        let data = fs::read(&file).expect("actual build identity bytes");
        digest.update((relative.len() as u64).to_le_bytes());
        digest.update(relative.as_bytes());
        digest.update((data.len() as u64).to_le_bytes());
        digest.update(data);
    }
    for (name, bytes) in host_configs {
        digest.update((name.len() as u64).to_le_bytes());
        digest.update(name.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    for name in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTUP_TOOLCHAIN",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_CFG_PANIC",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        let value = env::var(name).unwrap_or_default();
        digest.update((name.len() as u64).to_le_bytes());
        digest.update(name.as_bytes());
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    let compiler = Command::new(env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("-Vv")
        .output()
        .expect("actual compiler identity");
    assert!(compiler.status.success());
    digest.update(&compiler.stdout);
    let mut features: Vec<_> = env::vars()
        .filter_map(|(k, _)| k.strip_prefix("CARGO_FEATURE_").map(str::to_owned))
        .collect();
    features.sort();
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let graph = prepared_graph(
        workspace,
        std::str::from_utf8(&compiler.stdout).unwrap(),
        &target,
        &profile,
        &features.join(","),
    );
    let graph = match graph {
        Some(graph) => graph,
        None if env::var_os("CMVA_PREPARED_BUILD").is_some() => {
            panic!("supplied CMVA prepared build does not match the actual compiler, target, profile, features, inputs, environment or dependency graph")
        }
        None => String::new(), // Unbound builds remain diagnostic-only.
    };
    digest.update(graph.as_bytes());
    digest.update(target.as_bytes());
    digest.update(profile.as_bytes());
    println!("cargo:rustc-env=CMVA_BUILD_SHA256={:x}", digest.finalize());
    println!("cargo:rustc-env=CMVA_BUILD_TARGET={target}");
    println!("cargo:rustc-env=CMVA_BUILD_PROFILE={profile}");
    println!("cargo:rustc-env=CMVA_BUILD_FEATURES={}", features.join(","));
    println!("cargo:rustc-env=CMVA_BUILD_DEPENDENCY_GRAPH={graph}");
}
