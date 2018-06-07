use std::env;
use std::fs;
use std::io::{Write, Seek};

#[derive(Copy, Clone)]
pub enum LlamaFile {
    SdCardImg,
    NandImg,
    NandCid,
    AesKeyDb,
    Otp
}

fn make_filepath(filename: &str) -> String {
    format!("{}/.config/llama/{}", env::var("HOME").unwrap(), filename)
}

fn get_path(lf: LlamaFile) -> String {
    let filename = match lf {
        LlamaFile::SdCardImg => "sd.fat",
        LlamaFile::NandImg => "nand.bin",
        LlamaFile::NandCid => "nand-cid.bin",
        LlamaFile::AesKeyDb => "aeskeydb.bin",
        LlamaFile::Otp => "otp.bin"
    };
    make_filepath(filename)
}

pub fn open_file(lf: LlamaFile) -> Result<fs::File, String> {
    let path = get_path(lf);
    let res = fs::OpenOptions::new().read(true).write(true).open(path.as_str());
    match res {
        Ok(file) => Ok(file),
        Err(_) => Err(format!("Could not open file `{}`", path))
    }
}

pub fn create_file<F>(lf: LlamaFile, initializer: F) -> Result<fs::File, String>
    where F: FnOnce(&mut fs::File) {
        let path = get_path(lf);
        let res = fs::OpenOptions::new()
            .read(true).write(true)
            .create(true).truncate(true)
            .open(path.as_str());
    let mut file = match res {
        Ok(file) => file,
        Err(x) => return Err(format!("Could not create file `{}`; {:?}", path, x))
    };
    initializer(&mut file);
    Ok(file)
}