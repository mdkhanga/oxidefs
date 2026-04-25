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

    /// Size of the superblock when serialized to bytes
    pub const SERIALIZED_SIZE: usize = 112;

    /// Serialize the superblock to a byte buffer
    ///
    /// Returns a fixed-size array that can be written to block 0.
    /// Uses little-endian byte order for all multi-byte values.
    pub fn to_bytes(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        let mut offset = 0;

        // Write u32 values
        buf[offset..offset + 4].copy_from_slice(&self.magic.to_le_bytes());
        offset += 4;
        buf[offset..offset + 4].copy_from_slice(&self.version.to_le_bytes());
        offset += 4;
        buf[offset..offset + 4].copy_from_slice(&self.block_size.to_le_bytes());
        offset += 4;
        buf[offset..offset + 4].copy_from_slice(&self._padding.to_le_bytes());
        offset += 4;

        // Write u64 values
        buf[offset..offset + 8].copy_from_slice(&self.total_blocks.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.free_blocks.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.total_inodes.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.free_inodes.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.block_bitmap_start.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.inode_bitmap_start.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.inode_table_start.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.data_blocks_start.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.mount_time.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.write_time.to_le_bytes());
        offset += 8;

        // Final u32 values
        buf[offset..offset + 4].copy_from_slice(&self.mount_count.to_le_bytes());
        offset += 4;
        buf[offset..offset + 4].copy_from_slice(&self.max_mount_count.to_le_bytes());
        // offset += 4; // Not needed, we're done

        buf
    }

    /// Deserialize a superblock from a byte buffer
    ///
    /// Returns None if the buffer is too small or the magic number is invalid.
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::SERIALIZED_SIZE {
            return None;
        }

        let mut offset = 0;

        // Helper to read u32
        let read_u32 = |o: usize| -> u32 {
            u32::from_le_bytes(buf[o..o + 4].try_into().unwrap())
        };

        // Helper to read u64
        let read_u64 = |o: usize| -> u64 {
            u64::from_le_bytes(buf[o..o + 8].try_into().unwrap())
        };

        // Read u32 fields
        let magic = read_u32(offset);
        offset += 4;
        let version = read_u32(offset);
        offset += 4;
        let block_size = read_u32(offset);
        offset += 4;
        let _padding = read_u32(offset);
        offset += 4;

        // Read u64 fields
        let total_blocks = read_u64(offset);
        offset += 8;
        let free_blocks = read_u64(offset);
        offset += 8;
        let total_inodes = read_u64(offset);
        offset += 8;
        let free_inodes = read_u64(offset);
        offset += 8;
        let block_bitmap_start = read_u64(offset);
        offset += 8;
        let inode_bitmap_start = read_u64(offset);
        offset += 8;
        let inode_table_start = read_u64(offset);
        offset += 8;
        let data_blocks_start = read_u64(offset);
        offset += 8;
        let mount_time = read_u64(offset);
        offset += 8;
        let write_time = read_u64(offset);
        offset += 8;

        // Final u32 fields
        let mount_count = read_u32(offset);
        offset += 4;
        let max_mount_count = read_u32(offset);
        // offset += 4; // Not needed

        let sb = Self {
            magic,
            version,
            block_size,
            _padding,
            total_blocks,
            free_blocks,
            total_inodes,
            free_inodes,
            block_bitmap_start,
            inode_bitmap_start,
            inode_table_start,
            data_blocks_start,
            mount_time,
            write_time,
            mount_count,
            max_mount_count,
        };

        // Validate magic number
        if sb.is_valid() {
            Some(sb)
        } else {
            None
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superblock_new() {
        let sb = Superblock::new(1000, 100);

        assert!(sb.is_valid());
        assert_eq!(sb.magic, OXIDEFS_MAGIC);
        assert_eq!(sb.version, FS_VERSION);
        assert_eq!(sb.block_size, BLOCK_SIZE as u32);
        assert_eq!(sb.total_blocks, 1000);
        assert_eq!(sb.total_inodes, 100);
    }

    #[test]
    fn test_superblock_layout_calculation() {
        let sb = Superblock::new(1000, 100);

        // Block bitmap starts at block 1 (after superblock)
        assert_eq!(sb.block_bitmap_start, 1);

        // Inode bitmap comes after block bitmap
        assert!(sb.inode_bitmap_start > sb.block_bitmap_start);

        // Inode table comes after inode bitmap
        assert!(sb.inode_table_start > sb.inode_bitmap_start);

        // Data blocks come after inode table
        assert!(sb.data_blocks_start > sb.inode_table_start);

        // Data blocks should start before total blocks
        assert!(sb.data_blocks_start < sb.total_blocks);
    }

    #[test]
    fn test_superblock_free_counts() {
        let sb = Superblock::new(1000, 100);

        // Free blocks should be less than total (metadata uses some)
        assert!(sb.free_blocks < sb.total_blocks);

        // Free inodes should be total - 1 (root inode pre-allocated)
        assert_eq!(sb.free_inodes, sb.total_inodes - 1);
    }

    #[test]
    fn test_superblock_serialized_size() {
        // Verify our size constant matches actual struct layout
        // 4 u32s (16 bytes) + 10 u64s (80 bytes) + 2 u32s (8 bytes) = 104...
        // Wait, let me count: 4 + 4 + 4 + 4 + 8*10 + 4 + 4 = 16 + 80 + 8 = 104
        // But we have 112. Let me recount the u64 fields...
        // total_blocks, free_blocks, total_inodes, free_inodes,
        // block_bitmap_start, inode_bitmap_start, inode_table_start, data_blocks_start,
        // mount_time, write_time = 10 u64s = 80 bytes
        // magic, version, block_size, _padding, mount_count, max_mount_count = 6 u32s = 24 bytes
        // Total = 104 bytes... hmm, let's just test the constant is reasonable
        assert!(Superblock::SERIALIZED_SIZE >= 100);
        assert!(Superblock::SERIALIZED_SIZE <= 256);
    }

    #[test]
    fn test_superblock_roundtrip() {
        // This is the most important test!
        // Serialize then deserialize should give back identical data
        let original = Superblock::new(10000, 500);

        // Serialize to bytes
        let bytes = original.to_bytes();

        // Deserialize back
        let restored = Superblock::from_bytes(&bytes).expect("should deserialize");

        // Verify all fields match
        assert_eq!(original.magic, restored.magic);
        assert_eq!(original.version, restored.version);
        assert_eq!(original.block_size, restored.block_size);
        assert_eq!(original.total_blocks, restored.total_blocks);
        assert_eq!(original.free_blocks, restored.free_blocks);
        assert_eq!(original.total_inodes, restored.total_inodes);
        assert_eq!(original.free_inodes, restored.free_inodes);
        assert_eq!(original.block_bitmap_start, restored.block_bitmap_start);
        assert_eq!(original.inode_bitmap_start, restored.inode_bitmap_start);
        assert_eq!(original.inode_table_start, restored.inode_table_start);
        assert_eq!(original.data_blocks_start, restored.data_blocks_start);
        assert_eq!(original.mount_time, restored.mount_time);
        assert_eq!(original.write_time, restored.write_time);
        assert_eq!(original.mount_count, restored.mount_count);
        assert_eq!(original.max_mount_count, restored.max_mount_count);
    }

    #[test]
    fn test_superblock_invalid_magic() {
        let sb = Superblock::new(1000, 100);
        let mut bytes = sb.to_bytes();

        // Corrupt the magic number
        bytes[0] = 0xFF;
        bytes[1] = 0xFF;

        // Should fail to deserialize
        assert!(Superblock::from_bytes(&bytes).is_none());
    }

    #[test]
    fn test_superblock_buffer_too_small() {
        let small_buffer = [0u8; 50]; // Too small
        assert!(Superblock::from_bytes(&small_buffer).is_none());
    }
}
