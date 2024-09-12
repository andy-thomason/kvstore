#![allow(dead_code)]
use std::fmt::Write;

use crate::Storage;

const PAGE_SIZE : usize = 0x1000;

pub const FILE_MAGIC : [u8; 4] = *b"kv1f";
pub const LEAF_MAGIC : [u8; 4] = *b"kv1l";
pub const INDEX_MAGIC : [u8; 4] = *b"kv1i";

#[derive(Clone, Default)]
pub(crate) struct W64([u8; 8]);

#[derive(Debug)]
pub enum Error {
    Bad,
}

impl W64 {
    pub fn new(value: u64) -> Self {
        Self(value.to_be_bytes())
    }

    pub fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.0)
    }
}

#[repr(C)]
pub(crate) struct FileHeader {
    pub magic: [u8; 4],
    pub index_page: W64,
    pub first_free_page: W64,
    pub page_size: W64,
}

pub(crate) struct FilePage<'a, const PAGE_SIZE: usize>(&'a mut [u8; PAGE_SIZE]);

pub(crate) const NULL_PAGE : u64 = !0;
pub(crate) const MIN_PAGE_SIZE : usize = 32;

impl<'a, const PAGE_SIZE: usize> FilePage<'a, PAGE_SIZE> {
    /// Make a new file header page from scratch.
    pub fn new(buf: &'a mut [u8; PAGE_SIZE]) -> Self {
        let mut res = Self(buf);
        let hdr = res.header_mut();
        hdr.magic = FILE_MAGIC;
        hdr.first_free_page = W64::new(NULL_PAGE);
        hdr.index_page = W64::new(NULL_PAGE);
        hdr.page_size = W64::new(PAGE_SIZE as u64);
        assert!(PAGE_SIZE.is_power_of_two());
        assert!(PAGE_SIZE >= MIN_PAGE_SIZE);
        assert!(PAGE_SIZE >= std::mem::size_of::<FileHeader>());
        res
    }

    /// Wrap and check an existing file header page.
    pub fn from_buf(buf: &'a mut [u8; PAGE_SIZE]) -> Result<Self, Error> {
        let res = Self(buf);

        if res.header().magic != FILE_MAGIC {
            return Err(Error::Bad);
        }

        if res.header().page_size.as_u64() != PAGE_SIZE as u64 {
            return Err(Error::Bad);
        }
        
        assert!(PAGE_SIZE.is_power_of_two());
        assert!(PAGE_SIZE >= MIN_PAGE_SIZE);
        assert!(PAGE_SIZE >= std::mem::size_of::<FileHeader>());

        Ok(res)
    }

    /// Write the page to storage.
    pub fn write<W : Storage>(&self, w: &W, pos: u64) -> Result<(), Error> {
        Storage::write(w, pos, self.0).map_err(|_| Error::Bad)
    }

    /// Reference the header.
    pub fn header(&self) -> &FileHeader {
        let p = self.0.as_ptr() as *const FileHeader;
        unsafe { & *p }
    }

    /// Mutably reference the header.
    pub fn header_mut(&mut self) -> &mut FileHeader {
        let p = self.0.as_mut_ptr() as *mut FileHeader;
        unsafe { & mut *p }
    }
}

#[repr(C)]
pub(crate) struct LeafHeader {
    magic: [u8; 4],
    len: W64,
}

struct Hex<'a>(&'a [u8]);

impl<'a> std::fmt::Display for Hex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = b"0123456789abcdef";
        for b in self.0 {
            let b = *b as usize;
            f.write_char(hex[b/16] as char).unwrap();
            f.write_char(hex[b%16] as char).unwrap();
        }
        Ok(())
    }
}

#[test]
fn test() {
    let mut buf = [0; 32];
    let page = FilePage::<32>::new(&mut buf);
    assert_eq!(format!("{}", Hex(page.0)), "6b7673746f726520ffffffffffffffffffffffffffffffff0000000000000000");
}
