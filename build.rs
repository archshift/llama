extern crate bindgen;

use std::env;
use std::path;
use std::process;

fn main() {
    let base_dir = env::current_dir().unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let qml_dir = base_dir.join("llama-ui/qml");

    let status = process::Command::new("qmake")
        .current_dir(&out_dir)
        .arg(qml_dir)
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

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=dylib={}", "llamagui");

    bindgen::Builder::default()
        .header("llama-ui/qml/interop.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(path::PathBuf::from(out_dir).join("qml_interop.rs"))
        .expect("Couldn't write bindings!");
}