extern crate bindgen;

use std::env;
use std::fs;
use std::path;
use std::process;

fn exe_dir() -> path::PathBuf {
    let base_dir = env::current_dir().unwrap();
    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let mut exe_dir = base_dir.join("target");
    if host != target {
        exe_dir = exe_dir.join(target);
    }
    exe_dir = exe_dir.join(profile);
    exe_dir
}

fn to_lib_name(base_name: &str) -> String {
    let target = env::var("TARGET").unwrap();
    let (prefix, suffix) = if target.contains("apple") {
        ("lib", ".dylib")
    } else if target.contains("linux") {
        ("lib", ".so")
    } else {
        unimplemented!()
    };
    format!("{}{}{}", prefix, base_name, suffix)
}

fn main() {
    let base_dir = env::current_dir().unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let exe_dir = exe_dir();

    let qml_dir = base_dir.join("llama-ui/qml");

    let status = process::Command::new("qmake")
        .current_dir(&out_dir)
        .arg(&qml_dir)
        .spawn()
        .expect("failed to start qmake")
        .wait()
        .unwrap();

    assert!(status.success(), "failed to execute qmake");

    let status = process::Command::new("make")
        .current_dir(&out_dir)
        .spawn()
        .expect("failed to start make")
        .wait()
        .unwrap();

    assert!(status.success(), "failed to execute make");

    let lib_name = to_lib_name("llamagui");
    fs::copy(format!("{}/{}", out_dir, lib_name), exe_dir.join(lib_name)).unwrap();

    println!("cargo:rustc-link-search=native={}", exe_dir.as_os_str().to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib={}", "llamagui");
    println!("cargo:rerun-if-changed={}", qml_dir.as_os_str().to_str().unwrap());

    bindgen::Builder::default()
        .header("llama-ui/qml/interop.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(path::PathBuf::from(out_dir).join("qml_interop.rs"))
        .expect("Couldn't write bindings!");
}