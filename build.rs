use std::env;
use std::path::PathBuf;
use std::process;

fn find_deps_staticlib(name: &str) -> Vec<PathBuf> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut lib_dir = PathBuf::from(out_dir);
        lib_dir.pop();
        lib_dir.pop();
        lib_dir.pop();
        lib_dir.push("deps");

    let entry_filter = |path: &PathBuf| {
        let filename = path.file_name().unwrap()
                           .to_str().unwrap();
        filename.starts_with(&format!("lib{}", name))
            && filename.ends_with(".a")
    };

    let entries = lib_dir.read_dir().unwrap();
    entries.map(|file| file.unwrap().path())
           .filter(entry_filter)
           .collect()
}

fn main() {
    let base_dir = env::current_dir().unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let ref mut lgl_static_file = find_deps_staticlib("lglc")[0];
    let lgl_static_filename = lgl_static_file.file_name().unwrap()
                                             .to_str().unwrap();

    env::set_var("LGL_INC_DIR", base_dir.join("lglc").join("include"));
    env::set_var("LGL_LIB_DIR", lgl_static_file.parent().unwrap());
    env::set_var("LGL_LIB", &lgl_static_filename[3..lgl_static_filename.len()-2]);

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