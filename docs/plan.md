# OxideFS Build Plan

The full roadmap for OxideFS, broken into phases and concrete tasks.
Each phase ends with something you can run, test, or inspect.

**Legend:** ✅ done · 🔄 in progress / next · ⬜ not started · 💡 optional / stretch

---

## Phase 0 — Rust Foundations ✅

Core types and the first on-disk structure.

- ✅ `BlockNum` / `InodeNum` newtypes with conversions (`src/types.rs`)
- ✅ Core constants: `BLOCK_SIZE` (4096), `INODE_SIZE` (128), `OXIDEFS_MAGIC`, `ROOT_INODE`, `DIRECT_BLOCKS` (12)
- ✅ `FsError` enum + `FsResult` alias using `thiserror` (`src/error.rs`)
- ✅ `Superblock` struct with manual little-endian serialization, 112 bytes (`src/superblock.rs`)
- ✅ Layout calculation: block bitmap → inode bitmap → inode table → data blocks
- ✅ Round-trip and validation tests

## Phase 1 — Block Layer ✅

Read and write 4KB blocks to an image file; track free/used space.

- ✅ `BlockDevice` trait: `read_block`, `write_block`, `num_blocks` (`src/block_device.rs`)
- ✅ `FileBlockDevice` backed by an image file (`create`, `open`, range checks)
- ✅ `Bitmap` with `set` / `clear` / `is_set` / `allocate` (`src/bitmap.rs`)
- ✅ `mkfs-oxidefs`: writes the superblock and both bitmaps, reserves the root inode and the root directory's data block
- ✅ Superblock free counts match the on-disk bitmaps (fixed the off-by-one for the root data block)
- ✅ Manual inspection: hexdump of a fresh image (`xxd -a test.img`)

## Phase 2 — Inode Layer 🔄

File metadata: what a file *is*, separate from its name.

- ⬜ `Inode` struct: mode (type + permissions), uid, gid, size, link count, atime/mtime/ctime, 12 direct block pointers, 1 single-indirect pointer
- ⬜ Binary serialization of `Inode` to exactly `INODE_SIZE` (128) bytes, with round-trip tests
- ⬜ Locate inode N on disk: `block = inode_table_start + (N-1)*128 / 4096`, `offset = (N-1)*128 % 4096`
- ⬜ `read_inode` / `write_inode` (read-modify-write of the containing block)
- ⬜ Inode allocation and freeing via the inode bitmap, keeping `free_inodes` in the superblock in sync
- ⬜ Data block allocation and freeing via the block bitmap, keeping `free_blocks` in sync
- ⬜ Block mapping: file-relative block index → on-disk block (direct pointers first, then single-indirect)
- ⬜ `mkfs` writes a real root inode (directory, mode `0755`, link count 2, pointing at its reserved data block)
- ⬜ `mkfs` sets `write_time` in the superblock (currently 0)
- 💡 `MemoryBlockDevice` (a `Vec` of blocks) for faster tests that don't touch disk
- ⬜ Hexdump check: see the root inode at the start of the inode table

## Phase 3 — Directory Layer ⬜

Names → inode numbers. A directory is just a file whose contents are entries.

- ⬜ Directory entry format (ext2-style): inode number, record length, name length, file type, name
- ⬜ Serialize and parse entries within a data block
- ⬜ `mkfs` writes the root directory block with `.` and `..` (both pointing at inode 1)
- ⬜ `read_dir`: iterate all entries in a directory inode
- ⬜ `lookup(dir, name)` → inode number
- ⬜ Path resolution: `/a/b/c` → inode, walking from the root
- ⬜ Errors for missing paths and for "not a directory" mid-path
- 💡 A `debugfs`-style CLI (`oxidefs-inspect`) to print the superblock, inodes, and directory listings from an image

## Phase 4 — FUSE Integration (read-only) ⬜

Mount the image and use real tools on it.

- ⬜ Install macFUSE (macOS) / libfuse (Linux); build with `--features fuse`
- ⬜ Implement `fuser::Filesystem`: `init`, `lookup`, `getattr`, `readdir`, `open`, `read`
- ⬜ Map OxideFS inodes to FUSE attributes (`FileAttr`)
- ⬜ `mount-oxidefs <image> <mountpoint>` binary
- ⬜ Way to get test content into an image before the write path exists (e.g. `mkfs --populate <dir>`)
- ⬜ Milestone: `ls -la` and `cat` work on a mounted image

## Phase 5 — Write Path ⬜

Make it a real read-write filesystem.

- ⬜ `create` / `mknod`: allocate an inode, add a directory entry
- ⬜ `write`: allocate data blocks on demand, grow `size`
- ⬜ `mkdir` / `rmdir` (including `.`, `..`, and parent link counts)
- ⬜ `unlink`: remove an entry, decrement links, free the inode and blocks at zero links
- ⬜ `setattr`: truncate, chmod, chown, utimes
- ⬜ `rename` (same directory, then across directories)
- 💡 Hard links and symlinks
- ⬜ `fsck-oxidefs`: check that bitmaps, link counts, and free counts agree
- ⬜ Milestone: `cp`, `mkdir`, `rm`, `mv`, and `vim` work; the image survives unmount and remount

## Phase 6 — Advanced ⬜

Performance and durability.

- ⬜ Block cache (LRU) in front of `BlockDevice`
- ⬜ Double/triple indirect blocks for large files
- ⬜ Journaling (write-ahead log for metadata) and crash recovery
- 💡 Crash-consistency testing: kill the process mid-write, then run fsck
- 💡 Extents instead of block pointers (ext4-style)
- 💡 Benchmarks vs. tmpfs / the native filesystem

---

## Future Explorations 💡

Deep dives after the core filesystem works (see README for details).

- 💡 Write our own `ls`, `cat`, `cp`, `mkdir` in Rust
- 💡 Raw syscalls, bypassing libc
- 💡 A kernel filesystem module (Rust-for-Linux or C) registered with VFS directly
- 💡 A block device driver
