mod ctr9;

pub use self::ctr9::*;
use mem;

pub trait Loader {
    fn entrypoint(&self) -> u32;
    fn load(&self, controller: &mut mem::MemController);
}