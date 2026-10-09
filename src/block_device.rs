//! Block device abstraction
//!
//! Defines the `BlockDevice` trait for reading and writing fixed-size
//! blocks, plus a `FileBlockDevice` implementation backed by a regular
//! file (the filesystem image).

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::{FsError, FsResult};
use crate::types::{BlockNum, BLOCK_SIZE};

/// A single block's worth of bytes
pub type Block = [u8; BLOCK_SIZE as usize];

/// Abstraction over a block-addressable storage device
///
/// This lets the rest of the filesystem work in terms of `BlockNum`
/// instead of raw file offsets, and makes it possible to swap in a
/// different backing store later (e.g. an in-memory device for tests).
pub trait BlockDevice {
    /// Read the block at `block` into a buffer
    fn read_block(&mut self, block: BlockNum) -> FsResult<Block>;

    /// Write `data` to the block at `block`
    fn write_block(&mut self, block: BlockNum, data: &Block) -> FsResult<()>;

    /// Total number of blocks the device can address
    fn num_blocks(&self) -> u64;
}

/// A `BlockDevice` backed by a regular file (the filesystem image)
pub struct FileBlockDevice {
    file: File,
    num_blocks: u64,
}

impl FileBlockDevice {
    /// Create a new filesystem image file of the given size (in blocks),
    /// overwriting any existing file at `path`.
    pub fn create<P: AsRef<Path>>(path: P, num_blocks: u64) -> FsResult<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.set_len(num_blocks * BLOCK_SIZE)?;
        Ok(Self { file, num_blocks })
    }

    /// Open an existing filesystem image file
    ///
    /// The device's block count is derived from the file's length.
    pub fn open<P: AsRef<Path>>(path: P) -> FsResult<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let num_blocks = file.metadata()?.len() / BLOCK_SIZE;
        Ok(Self { file, num_blocks })
    }

    fn check_range(&self, block: BlockNum) -> FsResult<()> {
        if block.as_u64() >= self.num_blocks {
            Err(FsError::BlockOutOfRange(block.as_u64()))
        } else {
            Ok(())
        }
    }
}

impl BlockDevice for FileBlockDevice {
    fn read_block(&mut self, block: BlockNum) -> FsResult<Block> {
        self.check_range(block)?;
        let mut buf: Block = [0u8; BLOCK_SIZE as usize];
        self.file.seek(SeekFrom::Start(block.byte_offset()))?;
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn write_block(&mut self, block: BlockNum, data: &Block) -> FsResult<()> {
        self.check_range(block)?;
        self.file.seek(SeekFrom::Start(block.byte_offset()))?;
        self.file.write_all(data)?;
        Ok(())
    }

    fn num_blocks(&self) -> u64 {
        self.num_blocks
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_sets_file_length() {
        let tmp = NamedTempFile::new().unwrap();
        let dev = FileBlockDevice::create(tmp.path(), 10).unwrap();
        assert_eq!(dev.num_blocks(), 10);
        assert_eq!(tmp.path().metadata().unwrap().len(), 10 * BLOCK_SIZE);
    }

    #[test]
    fn test_write_then_read_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let mut dev = FileBlockDevice::create(tmp.path(), 4).unwrap();

        let mut data: Block = [0u8; BLOCK_SIZE as usize];
        data[0] = 0xAB;
        data[BLOCK_SIZE as usize - 1] = 0xCD;

        dev.write_block(BlockNum::new(2), &data).unwrap();
        let read_back = dev.read_block(BlockNum::new(2)).unwrap();

        assert_eq!(data, read_back);
    }

    #[test]
    fn test_unwritten_block_is_zeroed() {
        let tmp = NamedTempFile::new().unwrap();
        let mut dev = FileBlockDevice::create(tmp.path(), 4).unwrap();
        let block = dev.read_block(BlockNum::new(1)).unwrap();
        assert_eq!(block, [0u8; BLOCK_SIZE as usize]);
    }

    #[test]
    fn test_read_out_of_range() {
        let tmp = NamedTempFile::new().unwrap();
        let mut dev = FileBlockDevice::create(tmp.path(), 4).unwrap();
        let result = dev.read_block(BlockNum::new(4));
        assert!(matches!(result, Err(FsError::BlockOutOfRange(4))));
    }

    #[test]
    fn test_write_out_of_range() {
        let tmp = NamedTempFile::new().unwrap();
        let mut dev = FileBlockDevice::create(tmp.path(), 4).unwrap();
        let data: Block = [0u8; BLOCK_SIZE as usize];
        let result = dev.write_block(BlockNum::new(100), &data);
        assert!(matches!(result, Err(FsError::BlockOutOfRange(100))));
    }

    #[test]
    fn test_open_derives_num_blocks_from_file_length() {
        let tmp = NamedTempFile::new().unwrap();
        {
            FileBlockDevice::create(tmp.path(), 7).unwrap();
        }
        let dev = FileBlockDevice::open(tmp.path()).unwrap();
        assert_eq!(dev.num_blocks(), 7);
    }
}
