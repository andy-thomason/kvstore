use crate::{MapErr, Storage};

pub struct MemoryStorage(pub std::cell::RefCell<Vec<u8>>);

impl Storage for MemoryStorage {
    fn read(&self, pos: u64, buf: &mut [u8]) -> Result<(), crate::Error> {
        let vec = self.0.borrow();
        let pos : usize = pos.try_into().bad()?;
        buf.copy_from_slice(&vec[pos..pos+buf.len()]);
        Ok(())
    }

    fn write(&self, pos: u64, buf: &[u8]) -> Result<(), crate::Error> {
        let mut vec = self.0.borrow_mut();
        let pos : usize = pos.try_into().bad()?;
        let newlen = pos.saturating_add(buf.len());
        if newlen > vec.len() {
            vec.resize(newlen.try_into().bad()?, 0);
        }
        vec[pos..pos+buf.len()].copy_from_slice(buf);
        Ok(())
    }
}
