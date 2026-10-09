//! Bitmap for tracking allocation state
//!
//! Used for both the block bitmap and the inode bitmap: one bit per
//! item, packed 8 to a byte. A set bit means "in use".

/// A packed bitmap of a fixed length
#[derive(Debug, Clone)]
pub struct Bitmap {
    bits: Vec<u8>,
    len: u64,
}

impl Bitmap {
    /// Create a new bitmap with `len` bits, all initially clear (free)
    pub fn new(len: u64) -> Self {
        Self {
            bits: vec![0u8; Self::bytes_for(len)],
            len,
        }
    }

    /// Reconstruct a bitmap of `len` bits from its packed byte representation
    pub fn from_bytes(bytes: &[u8], len: u64) -> Self {
        let mut bits = bytes.to_vec();
        bits.resize(Self::bytes_for(len), 0);
        Self { bits, len }
    }

    /// The packed byte representation, suitable for writing to disk
    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }

    /// Number of bits this bitmap tracks
    pub fn len(&self) -> u64 {
        self.len
    }

    fn bytes_for(len: u64) -> usize {
        ((len + 7) / 8) as usize
    }

    /// Whether bit `index` is set (in use)
    ///
    /// # Panics
    /// Panics if `index >= len()`.
    pub fn is_set(&self, index: u64) -> bool {
        assert!(index < self.len, "bitmap index {index} out of range");
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;
        self.bits[byte] & (1 << bit) != 0
    }

    /// Mark bit `index` as set (in use)
    ///
    /// # Panics
    /// Panics if `index >= len()`.
    pub fn set(&mut self, index: u64) {
        assert!(index < self.len, "bitmap index {index} out of range");
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;
        self.bits[byte] |= 1 << bit;
    }

    /// Mark bit `index` as clear (free)
    ///
    /// # Panics
    /// Panics if `index >= len()`.
    pub fn clear(&mut self, index: u64) {
        assert!(index < self.len, "bitmap index {index} out of range");
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;
        self.bits[byte] &= !(1 << bit);
    }

    /// Find the first clear bit, set it, and return its index
    ///
    /// Returns `None` if every bit is already set.
    pub fn allocate(&mut self) -> Option<u64> {
        for i in 0..self.len {
            if !self.is_set(i) {
                self.set(i);
                return Some(i);
            }
        }
        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bitmap_all_clear() {
        let bm = Bitmap::new(20);
        for i in 0..20 {
            assert!(!bm.is_set(i));
        }
    }

    #[test]
    fn test_set_and_clear() {
        let mut bm = Bitmap::new(20);
        bm.set(5);
        assert!(bm.is_set(5));
        assert!(!bm.is_set(4));
        assert!(!bm.is_set(6));

        bm.clear(5);
        assert!(!bm.is_set(5));
    }

    #[test]
    fn test_allocate_finds_first_free() {
        let mut bm = Bitmap::new(8);
        bm.set(0);
        bm.set(1);

        let allocated = bm.allocate();
        assert_eq!(allocated, Some(2));
        assert!(bm.is_set(2));
    }

    #[test]
    fn test_allocate_returns_none_when_full() {
        let mut bm = Bitmap::new(4);
        for _ in 0..4 {
            bm.allocate().unwrap();
        }
        assert_eq!(bm.allocate(), None);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let mut bm = Bitmap::new(20);
        bm.set(0);
        bm.set(9);
        bm.set(17);

        let restored = Bitmap::from_bytes(bm.as_bytes(), 20);
        for i in 0..20 {
            assert_eq!(bm.is_set(i), restored.is_set(i));
        }
    }

    #[test]
    fn test_len_not_multiple_of_8() {
        let mut bm = Bitmap::new(13);
        assert_eq!(bm.as_bytes().len(), 2);
        bm.set(12);
        assert!(bm.is_set(12));
    }

    #[test]
    #[should_panic]
    fn test_out_of_range_panics() {
        let bm = Bitmap::new(8);
        bm.is_set(8);
    }
}
