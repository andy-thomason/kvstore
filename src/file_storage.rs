use std::io::{Read, Seek, Write};

use crate::{MapErr, Storage};

pub struct FileStorage(pub std::cell::RefCell<std::fs::File>);

impl Storage for FileStorage {
    fn read(&self, pos: u64, buf: &mut [u8]) -> Result<(), crate::Error> {
        self.0.borrow_mut().seek(std::io::SeekFrom::Start(pos)).bad()?;
        self.0.borrow_mut().read_exact(buf).bad()?;
        Ok(())
    }

    fn write(&self, pos: u64, buf: &[u8]) -> Result<(), crate::Error> {
        self.0.borrow_mut().seek(std::io::SeekFrom::Start(pos)).bad()?;
        self.0.borrow_mut().write_all(buf).bad()?;
        Ok(())
    }
}
