//! Error types for OxideFS
//!
//! This module defines all error types used throughout the filesystem.

use thiserror::Error;

/// All possible errors in OxideFS
#[derive(Debug, Error)]
pub enum FsError {
    /// I/O error from the underlying storage
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Filesystem has invalid magic number (not an OxideFS image)
    #[error("invalid filesystem magic number")]
    InvalidMagic,

    /// Requested inode does not exist
    #[error("inode {0} not found")]
    InodeNotFound(u64),

    /// Requested block does not exist
    #[error("block {0} out of range")]
    BlockOutOfRange(u64),

    /// No free blocks available
    #[error("no free blocks available")]
    NoFreeBlocks,

    /// No free inodes available
    #[error("no free inodes available")]
    NoFreeInodes,

    /// File or directory not found
    #[error("not found: {0}")]
    NotFound(String),

    /// Entry already exists
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Not a directory
    #[error("not a directory")]
    NotADirectory,

    /// Is a directory (when expecting a file)
    #[error("is a directory")]
    IsADirectory,

    /// Directory is not empty
    #[error("directory not empty")]
    DirectoryNotEmpty,
}

/// Convenience type alias for Results with FsError
pub type FsResult<T> = Result<T, FsError>;
