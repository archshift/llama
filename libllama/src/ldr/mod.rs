mod ctr9;
mod firm;

pub use self::ctr9::*;
pub use self::firm::*;
use mem;

pub trait Loader {
    fn entrypoint9(&self) -> u32;
    fn entrypoint11(&self) -> u32;
    fn load9(&self, controller: &mut mem::MemController);
    fn load11(&self, controller: &mut mem::MemController);
}

use std::path::Path;

pub fn make_loader(path: &Path) -> Box<dyn Loader> {
    match path.extension().and_then(|x| x.to_str()) {
        Some("ctr9") => Box::new(Ctr9Loader::from_folder(path).unwrap()),
        Some("firm") => Box::new(FirmLoader::from_file(path).unwrap()),
        Some(x) => panic!("Attempted to load with unknown extension {:?}", x),
        None => panic!("Attempted to load ambiguous file {:?}", path)
    }
}
