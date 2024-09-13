#![allow(dead_code)]
use core::cmp::Ordering;
use std::{fmt::Write, mem::size_of, marker::PhantomData};

use crate::Storage;

const PAGE_SIZE : usize = 0x1000;

pub(crate) trait Magic : Sized {
    const MAGIC_VALUE : [u8; 4];

    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Magic for FileHeaderV1 {
    const MAGIC_VALUE : [u8; 4] = *b"kv1f";
}

impl Magic for IndexHeaderV1 {
    const MAGIC_VALUE : [u8; 4] = *b"kv1i";
}

impl Magic for LeafHeaderV1 {
    const MAGIC_VALUE : [u8; 4] = *b"kv1l";
}

#[derive(Clone, Default)]
pub(crate) struct W64([u8; 8]);

#[derive(Debug)]
pub enum Error {
    Bad,
}

trait MapErr<T, E1> : Into<Result<T, E1>> {
    fn bad(self) -> Result<T, Error> {
        self.into().map_err(|_| Error::Bad)
    }
}

impl<T, E1> MapErr<T, E1> for Result<T, E1> {}

impl W64 {
    pub fn new(value: u64) -> Self {
        Self(value.to_be_bytes())
    }

    pub fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.0)
    }
}

#[repr(C)]
pub(crate) struct FileHeaderV1 {
    pub magic: [u8; 4],
    pub index_page: W64,
    pub first_free_page: W64,
    pub page_size: W64,
}

pub(crate) struct Page<'a, const PAGE_SIZE: usize, Header>{
    buf: &'a mut [u8; PAGE_SIZE],
    marker: PhantomData<Header>,
}

pub(crate) const NULL_PAGE : u64 = !0;
pub(crate) const MIN_PAGE_SIZE : usize = 32;
pub(crate) const MAX_PAGE_SIZE : usize = 0x10000;

impl<'a, const PAGE_SIZE: usize, Header> Page<'a, PAGE_SIZE, Header> {
    /// Write the page to storage.
    pub fn write<W : Storage>(&self, w: &W, pos: u64) -> Result<(), Error> {
        Storage::write(w, pos, self.buf).map_err(|_| Error::Bad)
    }

    /// Reference the header.
    pub fn header(&self) -> &Header {
        let p = self.buf.as_ptr() as *const Header;
        unsafe { & *p }
    }

    /// Mutably reference the header.
    pub fn header_mut(&mut self) -> &mut Header {
        let p = self.buf.as_mut_ptr() as *mut Header;
        unsafe { & mut *p }
    }

    pub fn check(&self) {
        assert!(PAGE_SIZE.is_power_of_two());
        assert!(PAGE_SIZE >= MIN_PAGE_SIZE);
        assert!(PAGE_SIZE >= FileHeaderV1::size());
    }
}

impl<'a, const PAGE_SIZE: usize> Page<'a, PAGE_SIZE, FileHeaderV1> {
    /// Make a new file header page from scratch.
    pub fn new(buf: &'a mut [u8; PAGE_SIZE]) -> Self {
        let mut res = Self { buf, marker: PhantomData };
        let hdr = res.header_mut();
        hdr.magic = FileHeaderV1::MAGIC_VALUE;
        hdr.first_free_page = W64::new(NULL_PAGE);
        hdr.index_page = W64::new(NULL_PAGE);
        hdr.page_size = W64::new(PAGE_SIZE as u64);
        res.check();
        res
    }

    /// Wrap and check an existing file header page.
    pub fn from_buf(buf: &'a mut [u8; PAGE_SIZE]) -> Result<Self, Error> {
        let res = Self { buf, marker: PhantomData };

        if res.header().magic != FileHeaderV1::MAGIC_VALUE {
            return Err(Error::Bad);
        }

        if res.header().page_size.as_u64() != PAGE_SIZE as u64 {
            return Err(Error::Bad);
        }
        
        res.check();
        Ok(res)
    }
}

#[repr(C)]
pub(crate) struct LeafHeaderV1 {
    magic: [u8; 4],
    len: W64,
}

#[repr(C)]
pub(crate) struct IndexHeaderV1 {
    magic: [u8; 4],
    len: W64,
}

impl<'a, const PAGE_SIZE: usize> Page<'a, PAGE_SIZE, IndexHeaderV1> {
    /// Make a new file header page from scratch.
    pub fn new(buf: &'a mut [u8; PAGE_SIZE]) -> Self {
        let mut res = Self { buf, marker: PhantomData };
        let hdr = res.header_mut();
        hdr.magic = IndexHeaderV1::MAGIC_VALUE;
        res.check();
        res
    }

    /// Wrap and check an existing file header page.
    pub fn from_buf(buf: &'a mut [u8; PAGE_SIZE]) -> Result<Self, Error> {
        let res = Self { buf, marker: PhantomData };

        if res.header().magic != IndexHeaderV1::MAGIC_VALUE {
            return Err(Error::Bad);
        }

        res.check();
        Ok(res)
    }

    pub fn get(&self, key: &[u8], res: &mut [u8]) -> Result<Option<usize>, Error> {
        todo!();
    }
}

impl<'a, const PAGE_SIZE: usize> Page<'a, PAGE_SIZE, LeafHeaderV1> {
    /// Make a new file header page from scratch.
    pub fn new(buf: &'a mut [u8; PAGE_SIZE]) -> Self {
        let mut res = Self { buf, marker: PhantomData };
        let hdr = res.header_mut();
        hdr.magic = LeafHeaderV1::MAGIC_VALUE;
        res.check();
        res
    }

    /// Wrap and check an existing file header page.
    pub fn from_buf(buf: &'a mut [u8; PAGE_SIZE]) -> Result<Self, Error> {
        let res = Self { buf, marker: PhantomData };

        if res.header().magic != LeafHeaderV1::MAGIC_VALUE {
            return Err(Error::Bad);
        }

        res.check();
        Ok(res)
    }

    pub fn get(&self, key: &[u8], res: &mut [u8]) -> Result<Option<usize>, Error> {
        let hdr = self.header();
        let len = usize::try_from(hdr.len.as_u64()).bad()?;
        let hdr_size = size_of::<LeafHeaderV1>();
        let keys_end = hdr_size+len*2+2;
        let values_end = keys_end+len*2+2;
        if len > PAGE_SIZE/4 || values_end > PAGE_SIZE {
            return Err(Error::Bad);
        }
        let key_offsets = &self.buf[hdr_size..keys_end];
        let value_offsets = &self.buf[keys_end..values_end];
        let bytes = &self.buf[values_end..PAGE_SIZE];
        let lower = partition(key, bytes, key_offsets, Ordering::Less);
        let upper = partition(key, bytes, key_offsets, Ordering::Greater);
        todo!();
    }
}

pub fn partition(key: &[u8], bytes: &[u8], key_offsets: &[u8], tie_break: Ordering) -> usize {
    let mut size = key_offsets.len()/2-1;
    let mut left = 0;
    let mut right = size;
    let ptr = key_offsets.as_ptr();
    while left < right {
        let mid = left + size / 2;

        let cmp = unsafe {
            let kb = get_u16(ptr.offset(mid as isize*2+0));
            let ke = get_u16(ptr.offset(mid as isize*2+2));
            bytes[kb..ke].cmp(key)
        };

        use Ordering::*;
        let cmp = if cmp == Equal { tie_break } else { cmp };
        left = if cmp == Less { mid + 1 } else { left };
        right = if cmp == Greater { mid } else { right };

        size = right - left;
    }
    left
}

unsafe fn get_u16(ptr: *const u8) -> usize {
    u16::from_be_bytes(
        [
            *ptr, *ptr.offset(1)
        ]
    ) as usize
}

struct Hex<'a>(&'a [u8]);

impl<'a> core::fmt::Display for Hex<'a> {
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

pub type FilePage<'a, const PAGE_SIZE: usize> = Page<'a, PAGE_SIZE, FileHeaderV1>;
pub type IndexPage<'a, const PAGE_SIZE: usize> = Page<'a, PAGE_SIZE, IndexHeaderV1>;
pub type LeafPage<'a, const PAGE_SIZE: usize> = Page<'a, PAGE_SIZE, LeafHeaderV1>;

#[test]
fn test() {
    let mut buf = [0; 32];
    let page = Page::<32, FileHeaderV1>::new(&mut buf);
    assert_eq!(format!("{}", Hex(page.buf)), "6b7673746f726520ffffffffffffffffffffffffffffffff0000000000000000");
}
