use std::env;
use std::process::Command;
use std::fs::File;
use std::path::Path;

fn main() {
    let decoders = [
        "src/cpu/arm.decoder",
        "src/cpu/thumb.decoder"
    ];

    for decoder in decoders.iter() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let filename = Path::new(decoder).file_name().unwrap();
        let out = format!("{}/{}.rs", out_dir, filename.to_os_string().to_str().unwrap());

        Command::new("python3")
            .arg("tools/decoder-gen/decoder-gen.py")
            .arg(decoder)
            .stdout(File::create(out).unwrap())
            .spawn()
            .expect("failed to execute child");
        
        println!("cargo:rerun-if-changed={}", decoder);
    }
}