use file_spec::FilePage;
use memory_storage::MemoryStorage;

mod file_spec;

#[cfg(feature="std")]
pub mod file_storage;

#[cfg(feature="std")]
pub mod memory_storage;

pub trait Storage {
    fn read(&self, pos: u64, buf: &mut [u8]) -> Result<(), Error>;
    fn write(&self, pos: u64, buf: &[u8]) -> Result<(), Error>;
}

trait MapErr<T, E1> : Into<Result<T, E1>> {
    fn bad(self) -> Result<T, Error> {
        self.into().map_err(|_| Error::Bad)
    }
}

impl<T, E1> MapErr<T, E1> for Result<T, E1> {}

#[derive(Debug, PartialEq)]
pub enum Error {
    Bad
}

pub trait Get {
    fn get(&mut self, key: &[u8], buf: &mut [u8]) -> Result<Option<usize>, Error>;
}

pub trait Set {
    fn set(&mut self, key: &[u8], buf: &[u8]) -> Result<(), Error>;
}

pub struct KvStore<S : Storage, const PAGE_SIZE: usize> {
    storage: S,
}

#[cfg(feature="std")]
impl<const PAGE_SIZE: usize> KvStore<file_storage::FileStorage, PAGE_SIZE> {
    pub fn create<P : AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        assert!(PAGE_SIZE.is_power_of_two() && PAGE_SIZE >= 4096);
        let storage = file_storage::FileStorage(std::cell::RefCell::new(std::fs::File::create(path).bad()?));
        let mut buf = [0; PAGE_SIZE];
        let file_page = file_spec::FilePage::new(&mut buf);
        file_page.write(&storage, 0).bad()?;
        Ok(Self {
            storage,
        })
    }

}

#[cfg(feature="std")]
impl<const PAGE_SIZE: usize> KvStore<memory_storage::MemoryStorage, PAGE_SIZE> {
    pub fn in_memory(capacity: usize) -> Result<Self, Error> {
        assert!(PAGE_SIZE.is_power_of_two() && PAGE_SIZE >= 4096);
        let vec = Vec::with_capacity(capacity);
        let storage = MemoryStorage(std::cell::RefCell::new(vec));
        let mut buf = [0; PAGE_SIZE];
        let file_page = file_spec::FilePage::new(&mut buf);
        file_page.write(&storage, 0).bad()?;
        Ok(Self {
            storage,
        })
    }
}

impl<const PAGE_SIZE: usize, S: Storage> KvStore<S, PAGE_SIZE> {
    fn read(&mut self, page: u64, buf: &mut [u8; PAGE_SIZE]) -> Result<(), Error> {
        self.storage.read(page * (PAGE_SIZE as u64), buf)?;
        Ok(())
    }
}

impl<S: Storage, const PAGE_SIZE: usize> Get for KvStore<S, PAGE_SIZE> {
    fn get(&mut self, key: &[u8], buf: &mut [u8]) -> Result<Option<usize>, Error> {
        let mut buf = [0; PAGE_SIZE];
        self.read(0, &mut buf)?;
        let file_page = FilePage::from_buf(&mut buf).bad()?;
        let file_hdr = file_page.header();

        if file_hdr.index_page.as_u64() == file_spec::NULL_PAGE {
            return Ok(None);
        }

        self.read(file_hdr.index_page.as_u64(), &mut buf)?;
        Ok(None)
    }
}

impl<S: Storage, const PAGE_SIZE: usize> Set for KvStore<S, PAGE_SIZE> {
    fn set(&mut self, key: &[u8], buf: &[u8]) -> Result<(), Error> {
        todo!()
    }
}


#[cfg(test)]
mod test {
    use crate::{Get, KvStore, Set};

    #[test]
    fn smoke() {
        let mut kv = KvStore::<_, 0x1000>::in_memory(0x10000).unwrap();

        assert_eq!(kv.set(b"abc", b"def"), Ok(()));
        let mut rbuf = [0; 4];
        assert_eq!(kv.get(b"abc", &mut rbuf), Ok(Some(3)));
    }
}
