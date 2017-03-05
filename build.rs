use std::env;
use std::path::{Path, PathBuf};
use std::process;

enum Profile {
    Release,
    Debug,
}

impl Profile {
    fn from_str(name: &str) -> Profile {
        match name {
            "release" => Profile::Release,
            "debug" => Profile::Debug,
            unk => panic!("Unknown build profile '{}'!", unk)
        }
    }
}

struct StaticCrateLib {
    pub lib_dir: PathBuf,
    pub lib_name: String,
}

fn build_static_crate(crate_dir: &Path, crate_name: &str) -> Result<StaticCrateLib, ()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();

    let lib_out_dir = PathBuf::from(&out_dir).join(crate_name);

    let mut cmd = process::Command::new("cargo");
    cmd.current_dir(crate_dir)
        .arg("build")
        .env("CARGO_TARGET_DIR", &lib_out_dir);

    if let Profile::Release = Profile::from_str(&profile) {
        cmd.arg("--release");
    }

    let status = cmd.spawn()
        .expect("failed to start cargo")
        .wait()
        .unwrap();

    assert!(status.success(), "failed to execute cargo");

    Ok(StaticCrateLib {
        lib_dir: lib_out_dir.join(profile),
        lib_name: crate_name.to_string(),
    })
}

fn main() {
    let base_dir = env::current_dir().unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let lglc_dir = PathBuf::from(&base_dir).join("lglc");
    let lib_desc = build_static_crate(&lglc_dir, "lglc").unwrap();

    env::set_var("LGL_LIB_DIR", lib_desc.lib_dir);
    env::set_var("LGL_LIB", lib_desc.lib_name);
    env::set_var("LGL_INC_DIR", lglc_dir.join("include"));

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
}