extern crate json;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use ldr;
use hwcore;
use mem;
use utils;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Json(json::Error);
    }

    errors {
        JsonItemError(item: String, filename: String) {
            description("invalid or missing json item")
            display("invalid or missing item `{}` in file {}", item, filename)
        }
    }
}

const DESC_FILENAME: &'static str = "desc.json";

pub struct Ctr9Loader {
    path: PathBuf,
    desc: Desc,
}

impl Ctr9Loader {
    pub fn from_folder(path_str: &str) -> Result<Ctr9Loader> {
        let path = Path::new(path_str);
        let json = Ctr9Loader::load_desc_json(&path)?;

        Ok(Ctr9Loader {
            path: path.to_path_buf(),
            desc: Desc::from_json(&json)?,
        })
    }

    fn load_desc_json(path: &Path) -> Result<json::JsonValue> {
        let mut desc = File::open(path.join(DESC_FILENAME))?;
        let mut desc_str = String::new();
        desc.read_to_string(&mut desc_str)?;
        Ok(json::parse(&desc_str)?)
    }
}

impl ldr::Loader for Ctr9Loader {
    fn entrypoint(&self) -> u32 {
        self.desc.entrypoint
    }

    fn load(&self, controller: &mut mem::MemController) {
        for binfile in self.desc.binfiles.iter() {
            let mut file = File::open(self.path.join(&binfile.bin)).unwrap();
            let mut vaddr = binfile.vaddr;

            let mut read_buf = [0u8; 1024];
            loop {
                let size = file.read(&mut read_buf).unwrap();
                if size == 0 { break; }
                controller.write_buf(vaddr, &read_buf[0..size]);
                vaddr += size as u32;
            }
        }
    }

    fn arm11_state(&self) -> hwcore::Arm11State {
        self.desc.arm11_state
    }
}


struct Desc {
    entrypoint: u32,
    binfiles: Vec<DescBinfile>,
    arm11_state: hwcore::Arm11State,
}

impl Desc {
    fn from_json(json: &json::JsonValue) -> Result<Desc> {
        let entrypoint_str = json["entryPoint"].as_str()
            .ok_or(ErrorKind::JsonItemError("entryPoint".to_owned(), DESC_FILENAME.to_owned()));
        let entrypoint = utils::from_hex(entrypoint_str?)
            .chain_err(|| ErrorKind::JsonItemError("entryPoint".to_owned(), DESC_FILENAME.to_owned()));

        // Load binfiles array into vec, make sure >1 binfile exists
        let mut binfiles = Vec::new();
        for binfile in json["binFiles"].members() {
            binfiles.push(DescBinfile::from_json(binfile)?);
        }
        if binfiles.len() == 0 {
            bail!(ErrorKind::JsonItemError("binfiles[]".to_owned(), DESC_FILENAME.to_owned()))
        }

        let arm11_state_str = json["arm11State"].as_str();
        let arm11_state = match arm11_state_str {
            Some("bootSync") => Ok(hwcore::Arm11State::BootSync),
            Some("kernelSync") => Ok(hwcore::Arm11State::KernelSync),
            Some("none") | None => Ok(hwcore::Arm11State::None),
            Some(_) => Err(ErrorKind::JsonItemError("arm11State".to_owned(), DESC_FILENAME.to_owned()))
        };

        Ok(Desc {
            entrypoint: entrypoint?,
            binfiles: binfiles,
            arm11_state: arm11_state?
        })
    }
}

struct DescBinfile {
    bin: String,
    vaddr: u32,
}

impl DescBinfile {
    fn from_json(json: &json::JsonValue) -> Result<DescBinfile> {
        let bin = json["bin"].as_str()
            .ok_or(ErrorKind::JsonItemError("binfiles[].bin".to_owned(), DESC_FILENAME.to_owned()));
        let vaddr_str = json["vAddr"].as_str()
            .ok_or(ErrorKind::JsonItemError("binfiles[].vAddr".to_owned(), DESC_FILENAME.to_owned()));
        let vaddr = utils::from_hex(vaddr_str?)
            .chain_err(|| ErrorKind::JsonItemError("binfiles[].vAddr".to_owned(), DESC_FILENAME.to_owned()));

        Ok(DescBinfile {
            bin: bin?.to_owned(),
            vaddr: vaddr?,
        })
    }
}