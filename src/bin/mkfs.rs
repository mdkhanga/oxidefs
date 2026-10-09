//! mkfs.oxidefs - Creates a new OxideFS filesystem image
//!
//! Usage: mkfs-oxidefs <image_path> <size>
//! Example: mkfs-oxidefs myfs.img 100M

use std::env;
use std::process::ExitCode;

use oxidefs::{Bitmap, BlockDevice, BlockNum, FileBlockDevice, Superblock, BLOCK_SIZE};

/// Ratio used to size the inode table: one inode per this many bytes of
/// filesystem space. Matches ext2's traditional default.
const BYTES_PER_INODE: u64 = 16 * 1024;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <image_path> <size>", args[0]);
        eprintln!("Example: {} myfs.img 100M", args[0]);
        return ExitCode::FAILURE;
    }

    let image_path = &args[1];
    let size_bytes = match parse_size(&args[2]) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("invalid size '{}': {e}", args[2]);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = make_filesystem(image_path, size_bytes) {
        eprintln!("mkfs-oxidefs: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn make_filesystem(image_path: &str, size_bytes: u64) -> oxidefs::FsResult<()> {
    let num_blocks = size_bytes / BLOCK_SIZE;
    let total_inodes = (size_bytes / BYTES_PER_INODE).max(16);

    let sb = Superblock::new(num_blocks, total_inodes);
    let mut dev = FileBlockDevice::create(image_path, num_blocks)?;

    // Block 0: superblock, zero-padded to a full block.
    let mut sb_block = [0u8; BLOCK_SIZE as usize];
    sb_block[..Superblock::SERIALIZED_SIZE].copy_from_slice(&sb.to_bytes());
    dev.write_block(BlockNum::new(0), &sb_block)?;

    // Block bitmap: every block before data_blocks_start is metadata and
    // is marked used, plus one data block reserved for the root directory.
    let mut block_bitmap = Bitmap::new(num_blocks);
    for i in 0..sb.data_blocks_start {
        block_bitmap.set(i);
    }
    block_bitmap.set(sb.data_blocks_start); // root directory's data block
    write_bitmap(&mut dev, sb.block_bitmap_start, &block_bitmap)?;

    // Inode bitmap: bit 0 represents inode 1 (root), which is pre-allocated.
    let mut inode_bitmap = Bitmap::new(total_inodes);
    inode_bitmap.set(0);
    write_bitmap(&mut dev, sb.inode_bitmap_start, &inode_bitmap)?;

    println!("Created OxideFS filesystem at '{image_path}'");
    println!("  size:          {size_bytes} bytes ({num_blocks} blocks)");
    println!("  inodes:        {total_inodes}");
    println!("  block bitmap:  block {}", sb.block_bitmap_start);
    println!("  inode bitmap:  block {}", sb.inode_bitmap_start);
    println!("  inode table:   block {}", sb.inode_table_start);
    println!("  data blocks:   block {}", sb.data_blocks_start);

    Ok(())
}

/// Write a bitmap's packed bytes to consecutive blocks starting at
/// `start_block`, zero-padding the final block.
fn write_bitmap(
    dev: &mut impl BlockDevice,
    start_block: u64,
    bitmap: &Bitmap,
) -> oxidefs::FsResult<()> {
    for (i, chunk) in bitmap.as_bytes().chunks(BLOCK_SIZE as usize).enumerate() {
        let mut block = [0u8; BLOCK_SIZE as usize];
        block[..chunk.len()].copy_from_slice(chunk);
        dev.write_block(BlockNum::new(start_block + i as u64), &block)?;
    }
    Ok(())
}

/// Parse a human size like "100M", "2G", "512K", or a plain byte count.
fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    let (digits, multiplier) = match s.chars().last() {
        Some('K' | 'k') => (&s[..s.len() - 1], 1024),
        Some('M' | 'm') => (&s[..s.len() - 1], 1024 * 1024),
        Some('G' | 'g') => (&s[..s.len() - 1], 1024 * 1024 * 1024),
        Some(_) => (s, 1),
        None => return Err("empty size".to_string()),
    };
    let n: u64 = digits
        .trim()
        .parse()
        .map_err(|_| format!("'{digits}' is not a number"))?;
    Ok(n * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_plain_bytes() {
        assert_eq!(parse_size("4096").unwrap(), 4096);
    }

    #[test]
    fn test_parse_size_suffixes() {
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("100M").unwrap(), 100 * 1024 * 1024);
        assert_eq!(parse_size("2G").unwrap(), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_size("1k").unwrap(), 1024);
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("abc").is_err());
        assert!(parse_size("").is_err());
    }
}
