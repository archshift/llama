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
    let (prefix, suffix) =
        if cfg!(target_os = "macos")
        { ("lib", ".dylib") }
        else if cfg!(target_os = "linux")
        { ("lib", ".so") }
        else if cfg!(target_os = "windows")
        { ("", ".dll") }
        else
        { unimplemented!() }
        ;
    format!("{}{}{}", prefix, base_name, suffix)
}

fn os_into(s: &path::Path) -> &str {
    s.as_os_str().to_str().unwrap()
}

fn make() -> &'static str {
    if cfg!(target_os = "windows") { "nmake" }
    else { "make" }
}

fn cmake_generator() -> &'static str {
    if cfg!(target_os = "windows") { "-GNMake Makefiles" }
    else { "-GUnix Makefiles" }
}

fn cmake_build_type() -> &'static str {
    match env::var("PROFILE").unwrap().as_ref() {
        "release" => "-DCMAKE_BUILD_TYPE=RelWithDebInfo",
        _ => "-DCMAKE_BUILD_TYPE=Debug",
    }
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
        .arg(cmake_build_type())
        .spawn()
        .expect("failed to start cmake")
        .wait()?;

    assert!(status.success(), "failed to execute cmake");

    let status = process::Command::new(make())
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
    println!("cargo:rerun-if-changed=build.rs");

    bindgen::Builder::default()
        .header("llama-ui/qml/interop.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("qml_interop.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
