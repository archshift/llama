extern crate bindgen;

use std::env;
use std::io;
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

fn os_into(s: &path::Path) -> &str {
    s.as_os_str().to_str().unwrap()
}

fn main() -> io::Result<()> {
    let base_dir = env::current_dir().unwrap();
    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    let exe_dir = exe_dir();

    let qml_dir = base_dir.join("llama-ui/qml");

    let status = process::Command::new("qmake")
        .current_dir(&out_dir)
        .arg(&qml_dir)
        .spawn()
        .expect("failed to start qmake")
        .wait()?;

    assert!(status.success(), "failed to execute qmake");

    let status = process::Command::new("make")
        .current_dir(&out_dir)
        .spawn()
        .expect("failed to start make")
        .wait()?;

    assert!(status.success(), "failed to execute make");

    let lib_name = to_lib_name("llamagui");
    fs::copy(out_dir.join(&lib_name), exe_dir.join(lib_name))?;

    println!("cargo:rustc-link-search=native={}", os_into(&exe_dir));
    println!("cargo:rustc-link-lib=dylib={}", "llamagui");

    for entry in fs::read_dir(qml_dir)? {
        println!("cargo:rerun-if-changed={}", os_into(&entry?.path()));
    }

    bindgen::Builder::default()
        .header("llama-ui/qml/interop.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("qml_interop.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
