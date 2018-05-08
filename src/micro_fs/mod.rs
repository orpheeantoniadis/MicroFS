mod create;

use std::mem;
use std::fs::File;
pub use self::create::*;

pub struct MicroFS {
    file: File,
    sb: SuperBlock
}
impl MicroFS {
    pub fn new() -> MicroFS {
        unsafe {
            MicroFS {
                file: mem::uninitialized(),
                sb: mem::uninitialized()
            }
        }
    }
}