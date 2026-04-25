//! Basic type definitions for OxideFS
//!
//! This module defines newtypes and constants that make our code
//! more readable and type-safe.

// =============================================================================
// Filesystem Constants
// =============================================================================

/// Block size in bytes (4KB - matches Linux page size)
pub const BLOCK_SIZE: u64 = 4096;

/// Magic number to identify OxideFS filesystems
/// "OxFS" in ASCII: 0x4F784653
pub const OXIDEFS_MAGIC: u32 = 0x4F784653;

/// Size of an inode structure on disk (128 bytes)
pub const INODE_SIZE: u64 = 128;

/// Root directory inode number (always 1, inode 0 is reserved/invalid)
pub const ROOT_INODE: u64 = 1;

/// Number of direct block pointers in an inode
pub const DIRECT_BLOCKS: usize = 12;

// =============================================================================
// Newtype Wrappers
// =============================================================================

/// Block number - identifies a block on disk (0-indexed)
///
/// Using a newtype instead of a raw u64 prevents accidentally mixing
/// block numbers with inode numbers or other integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockNum(pub u64);

/// Inode number - identifies an inode (1-indexed, 0 is invalid)
///
/// Inode 0 is reserved and indicates "no inode" (like NULL).
/// Inode 1 is always the root directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct InodeNum(pub u64);

// =============================================================================
// Newtype Implementations
// =============================================================================

impl BlockNum {
    /// Create a new block number
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    /// Get the raw u64 value
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Calculate the byte offset of this block in the device
    pub fn byte_offset(self) -> u64 {
        self.0 * BLOCK_SIZE
    }
}

impl InodeNum {
    /// Create a new inode number
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    /// Get the raw u64 value
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Check if this is a valid inode number (non-zero)
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }

    /// The root directory inode
    pub fn root() -> Self {
        Self(ROOT_INODE)
    }
}

// =============================================================================
// Display Implementations
// =============================================================================

// Allow displaying these types nicely with println!("{}", block)
impl std::fmt::Display for BlockNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "block#{}", self.0)
    }
}

impl std::fmt::Display for InodeNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inode#{}", self.0)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_num_new() {
        let block = BlockNum::new(42);
        assert_eq!(block.as_u64(), 42);
    }

    #[test]
    fn test_block_num_byte_offset() {
        let block = BlockNum::new(0);
        assert_eq!(block.byte_offset(), 0);

        let block = BlockNum::new(1);
        assert_eq!(block.byte_offset(), BLOCK_SIZE);

        let block = BlockNum::new(10);
        assert_eq!(block.byte_offset(), 10 * BLOCK_SIZE);
    }

    #[test]
    fn test_block_num_display() {
        let block = BlockNum::new(5);
        assert_eq!(format!("{}", block), "block#5");
    }

    #[test]
    fn test_inode_num_validity() {
        let invalid = InodeNum::new(0);
        assert!(!invalid.is_valid());

        let valid = InodeNum::new(1);
        assert!(valid.is_valid());
    }

    #[test]
    fn test_inode_num_root() {
        let root = InodeNum::root();
        assert_eq!(root.as_u64(), ROOT_INODE);
        assert!(root.is_valid());
    }

    #[test]
    fn test_constants() {
        assert_eq!(BLOCK_SIZE, 4096);
        assert_eq!(INODE_SIZE, 128);
        assert_eq!(DIRECT_BLOCKS, 12);
        assert_eq!(ROOT_INODE, 1);
        assert_eq!(OXIDEFS_MAGIC, 0x4F784653);
    }
}
