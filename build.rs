extern crate gcc;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build_dir = out_dir.join("build");
    let src_dir = env::current_dir().unwrap();

    if let Some(tcmalloc) = env::var_os("TCMALLOC_OVERRIDE") {
        let tcmalloc = PathBuf::from(tcmalloc);
        if let Some(parent) = tcmalloc.parent() {
            println!("cargo:rustc-link-search=native={}", parent.display());
        }
        let stem = tcmalloc.file_stem().unwrap().to_str().unwrap();
        let name = tcmalloc.file_name().unwrap().to_str().unwrap();
        let kind = if name.ends_with(".a") {"static"} else {"dylib"};
        println!("cargo:rustc-link-lib={}={}", kind, &stem[3..]);
        return
    }

    fs::create_dir_all(&build_dir).unwrap();

    if !target.contains("windows") || target.contains("windows-gnu") {
        build_on_nix(src_dir.as_path(), build_dir.as_path(), out_dir.as_path());
    } else {
        build_on_windows(src_dir.as_path(), build_dir.as_path(), out_dir.as_path());
    }
}

fn build_on_nix(src_dir: &Path, build_dir: &Path, out_dir: &Path) {
    let compiler = gcc::Config::new().get_compiler();
    let cflags = compiler.args().iter().map(|s| s.to_str().unwrap())
                         .collect::<Vec<_>>().join(" ");

    let mut cmd = Command::new("sh");
    cmd.arg(src_dir.join("gperftools/configure").to_str().unwrap())
       .current_dir(&build_dir)
       .env("CC", compiler.path())
       .env("EXTRA_CFLAGS", cflags)
       .arg(format!("--prefix={}", out_dir.display()))
       .arg("--with-pic=yes")
       .arg("--disable-shared")
       .arg("--disable-debugalloc");

    run(&mut cmd);
    run(Command::new("make")
                .current_dir(&build_dir)
                .arg("install-libLTLIBRARIES")
                .arg("-j").arg(env::var("NUM_JOBS").unwrap()));

    println!("cargo:root={}", out_dir.display());
    
    println!("cargo:rustc-link-lib=static=tcmalloc");

    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rustc-link-lib=unwind");
    
    println!("cargo:rustc-link-search=native={}/lib", out_dir.display());
}

fn build_on_windows(_: &Path, _: &Path, _: &Path) {
    unimplemented!()
}

fn run(cmd: &mut Command) -> &mut Command {
    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(e) => panic!("failed to execute command: {}", e),
    };
    if !status.success() {
        panic!("command did not execute successfully: {:?}\n\
                expected success, got: {}", cmd, status);
    }
    cmd
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let ptr = super::__rust_allocate(100, 8);
        assert!(!ptr.is_null());
        super::__rust_deallocate(ptr, 100, 8);
    }
}
