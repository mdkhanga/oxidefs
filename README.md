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
| 1 | Block layer - read/write blocks to image file | 🔄 Next |
| 2 | Inode layer - file metadata and allocation | ⬜ |
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

# (After Phase 4) Create and mount a filesystem
./target/debug/mkfs-oxidefs test.img 100M
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

**Phase 0 complete** - Foundation laid, not yet functional.

### What's Built
- Superblock structure with binary serialization (112 bytes)
- Type-safe `BlockNum` and `InodeNum` newtypes
- Error handling with custom `FsError` enum
- 13 unit tests passing (including serialization round-trip)
- Project scaffolding for `mkfs-oxidefs` and `mount-oxidefs` binaries

### What's Next
- **Phase 1: Block Layer** - Read/write 4KB blocks to a file image
- Implement `BlockDevice` trait for storage abstraction
- Block bitmap for tracking free/used blocks
- First hexdump of an actual filesystem image

### What's Not Working Yet
- Cannot create filesystem images (mkfs)
- Cannot mount (needs FUSE integration in Phase 4)
- No file or directory operations

## License

MIT

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
