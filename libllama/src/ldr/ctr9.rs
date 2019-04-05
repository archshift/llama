extern crate json;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use ldr;
use mem;
use utils;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error(non_std, no_from, msg_embedded)]
    JsonItemError(String),
    Io(::std::io::Error),
    Json(json::Error),
}

fn item_error(item: &str, filename: &str) -> ErrorKind {
    ErrorKind::JsonItemError(format!("invalid or missing item `{}` in file {}", item, filename))
}

const DESC_FILENAME: &'static str = "desc.json";

pub struct Ctr9Loader {
    path: PathBuf,
    desc: Desc,
}

impl Ctr9Loader {
    pub fn from_folder(path: &Path) -> Result<Ctr9Loader, ErrorKind> {
        let json = Ctr9Loader::load_desc_json(&path)?;

        Ok(Ctr9Loader {
            path: path.to_path_buf(),
            desc: Desc::from_json(&json)?,
        })
    }

    fn load_desc_json(path: &Path) -> Result<json::JsonValue, ErrorKind> {
        let mut desc = File::open(path.join(DESC_FILENAME))?;
        let mut desc_str = String::new();
        desc.read_to_string(&mut desc_str)?;
        Ok(json::parse(&desc_str)?)
    }
}

fn load_binfile(binfile: &DescBinfile, path: &Path, controller: &mut mem::MemController) {
    let mut file = File::open(path.join(&binfile.bin)).unwrap();
    let mut vaddr = binfile.vaddr;

    let mut read_buf = [0u8; 1024];
    loop {
        let size = file.read(&mut read_buf).unwrap();
        if size == 0 { break; }
        controller.write_buf(vaddr, &read_buf[0..size]);
        vaddr += size as u32;
    }
}

impl ldr::Loader for Ctr9Loader {
    fn entrypoint9(&self) -> u32 {
        self.desc.entrypoint
    }

    fn entrypoint11(&self) -> u32 {
        self.desc.entry11
    }

    fn load9(&self, controller: &mut mem::MemController) {
        for binfile in self.desc.binfiles.iter() {
            load_binfile(binfile, &self.path, controller);
        }
    }

    fn load11(&self, controller: &mut mem::MemController) {
        for binfile in self.desc.binfiles11.iter() {
            load_binfile(binfile, &self.path, controller);
        }
    }
}


struct Desc {
    entrypoint: u32,
    entry11: u32,
    binfiles: Vec<DescBinfile>,
    binfiles11: Vec<DescBinfile>,
}

impl Desc {
    fn from_json(json: &json::JsonValue) -> Result<Desc, ErrorKind> {
        let entrypoint_str = json["entryPoint"].as_str()
            .ok_or(item_error("entryPoint", DESC_FILENAME));
        let entrypoint = utils::from_hex(entrypoint_str?)
            .or(Err(item_error("entryPoint", DESC_FILENAME)));

        let entrypoint11_str = json["entryPoint11"].as_str()
            .ok_or(item_error("entryPoint11", DESC_FILENAME));
        let entrypoint11 = utils::from_hex(entrypoint11_str?)
            .or(Err(item_error("entryPoint11", DESC_FILENAME)));

        // Load binfiles arrays into vec, make sure >1 binfile exists
        let mut binfiles = Vec::new();
        for binfile in json["binFiles"].members() {
            binfiles.push(DescBinfile::from_json(binfile)?);
        }
        //if binfiles.len() == 0 {
        //    bail!(ErrorKind::JsonItemError("binfiles[]".to_owned(), DESC_FILENAME.to_owned()))
        //}

        let mut binfiles11 = Vec::new();
        for binfile in json["binFiles11"].members() {
            binfiles11.push(DescBinfile::from_json(binfile)?);
        }

        Ok(Desc {
            entrypoint: entrypoint?,
            entry11: entrypoint11?,
            binfiles: binfiles,
            binfiles11: binfiles11,
        })
    }
}

struct DescBinfile {
    bin: String,
    vaddr: u32,
}

impl DescBinfile {
    fn from_json(json: &json::JsonValue) -> Result<DescBinfile, ErrorKind> {
        let bin = json["bin"].as_str()
            .ok_or(item_error("binfiles[].bin", DESC_FILENAME));
        let vaddr_str = json["vAddr"].as_str()
            .ok_or(item_error("binfiles[].vAddr", DESC_FILENAME));
        let vaddr = utils::from_hex(vaddr_str?)
            .or(Err(item_error("binfiles[].vAddr", DESC_FILENAME)));

        Ok(DescBinfile {
            bin: bin?.to_owned(),
            vaddr: vaddr?,
        })
    }
}
