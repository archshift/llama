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

        let decoder_stat = Command::new("python3")
            .arg("tools/decoder-gen/decoder-gen.py")
            .arg(decoder)
            .stdout(File::create(&out).unwrap())
            .output()
            .expect("failed to execute child");

        if !decoder_stat.status.success() {
            use std::io::{stderr, Write};

            eprintln!("ERROR: decoder generation failed on {}!", out);
            eprintln!("Script stderr:");
            stderr().write_all(&decoder_stat.stderr).unwrap();
        }
        
        println!("cargo:rerun-if-changed={}", decoder);
    }
}
