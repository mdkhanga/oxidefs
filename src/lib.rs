//! OxideFS - A Linux-style filesystem implemented in Rust
//!
//! This crate provides an ext2-inspired filesystem that can be mounted
//! via FUSE (Filesystem in Userspace).
//!
//! # Architecture
//!
//! The filesystem is organized in layers:
//! - **Block layer**: Raw block I/O and allocation
//! - **Inode layer**: File metadata management
//! - **Directory layer**: Name-to-inode mapping
//! - **FUSE layer**: VFS interface implementation

// Module declarations - Rust requires explicit declaration of all modules
// We'll create these files as we implement each component

pub mod types;      // Basic type definitions (BlockNum, InodeNum, etc.)
pub mod error;      // Custom error types
pub mod superblock; // Filesystem superblock

// Re-export commonly used types at crate root for convenience
// This lets users write `oxidefs::BlockNum` instead of `oxidefs::types::BlockNum`
pub use types::*;
pub use error::{FsError, FsResult};
