# OxideFS

A Linux-style filesystem implemented in Rust, built from scratch as a learning project.

## What is this?

OxideFS is an **ext2-inspired FUSE filesystem** - a real, mountable filesystem that you can use with standard Unix tools (`ls`, `cat`, `mkdir`, etc.).

This project is an experiment in **vibe coding something non-trivial** while learning:
- **Rust** - systems programming, ownership, traits
- **Filesystem internals** - how files actually work at the block level

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                         │
│                   (ls, cat, vim, etc.)                       │
└─────────────────────────┬───────────────────────────────────┘
                          │ POSIX syscalls
┌─────────────────────────▼───────────────────────────────────┐
│                      Linux VFS                               │
└─────────────────────────┬───────────────────────────────────┘
                          │ FUSE protocol
┌─────────────────────────▼───────────────────────────────────┐
│                    OxideFS (this project)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ FUSE Layer  │──│  VFS Layer  │──│  Block Layer        │  │
│  │             │  │ (inodes,    │  │ (allocation,        │  │
│  │             │  │  dentries)  │  │  superblock)        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │ read/write
┌─────────────────────────▼───────────────────────────────────┐
│              Backing Store (file image)                      │
└─────────────────────────────────────────────────────────────┘
```

## On-Disk Layout

```
┌────────────┬────────────┬────────────┬────────────┬─────────────┐
│ Superblock │ Block      │ Inode      │ Inode      │ Data Blocks │
│ (block 0)  │ Bitmap     │ Bitmap     │ Table      │             │
└────────────┴────────────┴────────────┴────────────┴─────────────┘
```

- **Superblock**: Filesystem metadata (size, free counts, layout pointers)
- **Block Bitmap**: Tracks free/used data blocks (1 bit per block)
- **Inode Bitmap**: Tracks free/used inodes (1 bit per inode)
- **Inode Table**: File metadata (permissions, size, block pointers)
- **Data Blocks**: Actual file contents

## Build Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 0 | Rust foundations - types, superblock, serialization | ✅ Done |
| 1 | Block layer - read/write blocks to image file | ✅ Done |
| 2 | Inode layer - file metadata and allocation | 🔄 Next |
| 3 | Directory layer - name lookups, path resolution | ⬜ |
| 4 | FUSE integration - mount and use with real tools | ⬜ |
| 5 | Write path - create, write, delete files | ⬜ |
| 6 | Advanced - journaling, caching, etc. | ⬜ |

## Building

```bash
# Build the project
cargo build

# Run tests
cargo test

# Create a filesystem image
./target/debug/mkfs-oxidefs test.img 100M

# (After Phase 4) Mount it
mkdir -p /tmp/oxidefs
./target/debug/mount-oxidefs test.img /tmp/oxidefs
```

## Requirements

- Rust 1.70+
- macFUSE (macOS) or libfuse (Linux) - needed for Phase 4+

## Project Structure

```
src/
├── lib.rs           # Crate root
├── types.rs         # BlockNum, InodeNum, constants
├── error.rs         # FsError enum
├── superblock.rs    # Superblock structure + serialization
├── block_device.rs  # BlockDevice trait + file-backed implementation
├── bitmap.rs        # Free/used bitmap for blocks and inodes
└── bin/
    ├── mkfs.rs      # Creates filesystem images
    └── mount.rs     # Mounts filesystem via FUSE
```

## Learning Resources

This project was inspired by:
- "The Design of the UNIX Operating System" by Maurice Bach
- ext2 filesystem documentation
- The Rust Programming Language book

## Current Status

**Phase 1 complete** - `mkfs-oxidefs` creates real filesystem images.

### What's Built
- Superblock structure with binary serialization (112 bytes)
- Type-safe `BlockNum` and `InodeNum` newtypes
- Error handling with custom `FsError` enum
- `BlockDevice` trait for reading/writing 4KB blocks, with a `FileBlockDevice` backed by an image file
- `Bitmap` for tracking free/used blocks and inodes, with allocation
- Working `mkfs-oxidefs`: writes the superblock and both bitmaps, reserves the root inode and its data block
- 30 tests passing (including a check that superblock free counts match the on-disk bitmaps)

### What's Next
- **Phase 2: Inode Layer** - File metadata and allocation
- On-disk `Inode` struct with binary serialization
- Read/write inodes in the inode table
- Allocate/free inodes via the inode bitmap

### What's Not Working Yet
- Root inode is reserved but not yet written to the inode table
- Cannot mount (needs FUSE integration in Phase 4)
- No file or directory operations

## License

Apache 2.0

## Future Explorations

After completing the core filesystem, potential deep-dive projects:

### Build the Tools (Userspace)
Write our own `ls`, `cat`, `cp`, `mkdir` in Rust - understand how tools interact with filesystems via syscalls.

### Raw Syscalls (Skip libc)
Bypass the standard library and call the kernel directly using `syscall!` - learn what libc actually does.

### Kernel Filesystem Module
The real deal - write a Linux kernel module (in C or Rust) that registers with VFS directly. This is how ext4, xfs, and btrfs work. No FUSE, no userspace - pure kernel code.

```
┌─────────────────────────────────────────────┐
│ What we're building (FUSE)                  │
│ User tools → libc → VFS → FUSE → OxideFS   │
│                              (userspace)    │
├─────────────────────────────────────────────┤
│ The "full stack" (future)                   │
│ Our tools → raw syscalls → Our VFS module   │
│                              (kernel)       │
└─────────────────────────────────────────────┘
```

### Block Device Driver
Talk directly to disk hardware - requires understanding of device drivers, DMA, and hardware protocols.

---

*The FUSE approach gives 90% of the learning with 10% of the complexity. These explorations are for going deeper.*
