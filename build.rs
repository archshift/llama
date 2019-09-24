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

fn lib_files(base_name: &str) -> Vec<String> {
    if cfg!(target_os = "macos") {
        vec![ format!("lib{}.dylib", base_name) ]
    }
    else if cfg!(target_os = "linux") {
        vec![ format!("lib{}.so", base_name) ]
    }
    else if cfg!(target_os = "windows") {
        vec![
            format!("{}.dll", base_name),
            format!("{}.lib", base_name)
        ]
    }
    else {
        unimplemented!()
    }
}

fn copy_lib_files(base_name: &str, dst_dir: &path::Path) -> io::Result<()> {
    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_dir = if cfg!(target_os = "windows") {
        out_dir.clone().join(cmake_build_type())
    } else {
        out_dir.clone()
    };

    for lib_name in lib_files(base_name) {
        fs::copy(out_dir.join(&lib_name), dst_dir.join(lib_name))?;
    }

    Ok(())
}

fn os_into(s: &path::Path) -> &str {
    s.as_os_str().to_str().unwrap()
}

fn make() -> &'static [&'static str] {
    if cfg!(target_os = "windows") { &["cmake", "--build", "."] }
    else { &["make"] }
}

fn cmake_generator() -> &'static str {
    if cfg!(target_os = "windows") { "-GVisual Studio 16 2019" }
    else { "-GUnix Makefiles" }
}

fn cmake_build_type() -> &'static str {
    match env::var("PROFILE").unwrap().as_ref() {
        "release" => "RelWithDebInfo",
        _ => "Debug",
    }
}
fn cmake_build_type_arg() -> String {
    "-DCMAKE_BUILD_TYPE=".to_owned() + cmake_build_type()
}

fn main() -> io::Result<()> {
    let base_dir = env::current_dir().unwrap();
    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    let exe_dir = exe_dir();

    let qml_dir = base_dir.join("llama-ui/qml");

    let status = process::Command::new("cmake")
        .current_dir(&out_dir)
        .arg(&qml_dir)
        .arg(cmake_generator())
        .arg(cmake_build_type_arg())
        .spawn()
        .expect("failed to start cmake")
        .wait()?;

    assert!(status.success(), "failed to execute cmake");

    let make = make();
    let status = process::Command::new(make[0])
        .current_dir(&out_dir)
        .args(&make[1..])
        .spawn()
        .expect("failed to start make")
        .wait()?;

    assert!(status.success(), "failed to execute make");

    copy_lib_files("llamagui", &exe_dir)?;

    println!("cargo:rustc-link-search=native={}", os_into(&exe_dir));
    println!("cargo:rustc-link-lib=dylib={}", "llamagui");

    for entry in fs::read_dir(qml_dir)? {
        println!("cargo:rerun-if-changed={}", os_into(&entry?.path()));
    }
    println!("cargo:rerun-if-changed=build.rs");

    bindgen::Builder::default()
        .header("llama-ui/qml/interop.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("qml_interop.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
