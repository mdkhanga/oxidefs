//! Filesystem superblock
//!
//! The superblock contains metadata about the entire filesystem.
//! It is stored at block 0 and read on mount.

use crate::types::{BLOCK_SIZE, OXIDEFS_MAGIC};

/// Filesystem format version
pub const FS_VERSION: u32 = 1;

/// The superblock - filesystem metadata stored at block 0
///
/// This structure is serialized directly to disk, so we use:
/// - `#[repr(C)]` for predictable memory layout
/// - Fixed-size integer types (u32, u64) for portability
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Superblock {
    /// Magic number - must be OXIDEFS_MAGIC to identify valid filesystem
    pub magic: u32,

    /// Filesystem format version
    pub version: u32,

    /// Block size in bytes (always 4096 for us)
    pub block_size: u32,

    /// Padding to align next field to 8 bytes
    _padding: u32,

    /// Total number of blocks in the filesystem
    pub total_blocks: u64,

    /// Num of currently free blocks
    pub free_blocks: u64,

    /// Total number of inodes
    pub total_inodes: u64,

    /// Number of currently free inodes
    pub free_inodes: u64,

    /// Block number where the block bitmap starts
    pub block_bitmap_start: u64,

    /// Block number where the inode bitmap starts
    pub inode_bitmap_start: u64,

    /// Block number where the inode table starts
    pub inode_table_start: u64,

    /// Block number where data blocks start
    pub data_blocks_start: u64,

    /// Last mount time (Unix timestamp)
    pub mount_time: u64,

    /// Last write time (Unix timestamp)
    pub write_time: u64,

    /// Number of times mounted
    pub mount_count: u32,

    /// Maximum mounts before fsck recommended
    pub max_mount_count: u32,
}

impl Superblock {
    /// Create a new superblock for a filesystem of the given size
    ///
    /// # Arguments
    /// * `total_blocks` - Total number of blocks in the filesystem
    /// * `total_inodes` - Total number of inodes to allocate
    pub fn new(total_blocks: u64, total_inodes: u64) -> Self {
        // Calculate layout:
        // Block 0: Superblock
        // Block 1: Block bitmap (1 bit per block)
        // Block 2+: Inode bitmap (1 bit per inode)
        // Then: Inode table
        // Then: Data blocks

        // How many blocks needed for block bitmap?
        // Each block holds BLOCK_SIZE * 8 bits
        let bits_per_block = BLOCK_SIZE * 8;
        let block_bitmap_blocks = (total_blocks + bits_per_block - 1) / bits_per_block;

        // How many blocks needed for inode bitmap?
        let inode_bitmap_blocks = (total_inodes + bits_per_block - 1) / bits_per_block;

        // How many blocks for inode table?
        // Each inode is 128 bytes, so 32 inodes per block
        let inodes_per_block = BLOCK_SIZE / 128;
        let inode_table_blocks = (total_inodes + inodes_per_block - 1) / inodes_per_block;

        // Calculate start positions
        let block_bitmap_start = 1; // After superblock
        let inode_bitmap_start = block_bitmap_start + block_bitmap_blocks;
        let inode_table_start = inode_bitmap_start + inode_bitmap_blocks;
        let data_blocks_start = inode_table_start + inode_table_blocks;

        // Metadata blocks = everything before data
        let metadata_blocks = data_blocks_start;
        let free_blocks = total_blocks.saturating_sub(metadata_blocks);

        Self {
            magic: OXIDEFS_MAGIC,
            version: FS_VERSION,
            block_size: BLOCK_SIZE as u32,
            _padding: 0,
            total_blocks,
            free_blocks,
            total_inodes,
            free_inodes: total_inodes - 1, // Root inode is pre-allocated
            block_bitmap_start,
            inode_bitmap_start,
            inode_table_start,
            data_blocks_start,
            mount_time: 0,
            write_time: 0,
            mount_count: 0,
            max_mount_count: 20,
        }
    }

    /// Check if this superblock has a valid magic number
    pub fn is_valid(&self) -> bool {
        self.magic == OXIDEFS_MAGIC
    }
}
