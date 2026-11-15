//! The genome (decision 9): seven genes, two alleles each, fourteen bytes.
//!
//! - size, yield, temperament, hardiness: quantitative, each allele 0..255
//!   read as 0..1, expressed as the mean of the two;
//! - coat and pattern: an index into the species' list, the earlier dominant;
//! - morph: recessive, 1 is the morph allele; two copies show the morph.
//!
//! A wild animal's genome is rolled from its seed, so the same wild animal
//! is the same animal every time it is derived and when it is tamed.

use crate::fauna::hash2;

/// How many genes, and the bytes they take.
pub const GENES: usize = 7;
pub const BYTES: usize = GENES * 2;

/// Gene positions.
pub const SIZE: usize = 0;
pub const YIELD: usize = 1;
pub const TEMPERAMENT: usize = 2;
pub const HARDINESS: usize = 3;
pub const COAT: usize = 4;
pub const PATTERN: usize = 5;
pub const MORPH: usize = 6;

/// One wild allele in this many is the morph, so one wild animal in its
/// square (64) shows it (M9's rate).
pub const MORPH_ONE_IN: u64 = 8;

/// Fourteen bytes: gene `g`'s alleles at `2g` and `2g + 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Genome(pub [u8; BYTES]);

impl Default for Genome {
    /// Every quantitative gene at the middle, the first coat and pattern, no
    /// morph: the default a gene missing from an older record takes.
    fn default() -> Self {
        let mut bytes = [0u8; BYTES];
        for gene in [SIZE, YIELD, TEMPERAMENT, HARDINESS] {
            bytes[2 * gene] = 128;
            bytes[2 * gene + 1] = 128;
        }
        Self(bytes)
    }
}

impl Genome {
    /// A wild animal's genome, from its seed and its species' list lengths.
    pub fn roll(seed: u64, coats: usize, patterns: usize) -> Self {
        let mut bytes = [0u8; BYTES];
        for (k, byte) in bytes.iter_mut().enumerate() {
            let h = hash2(seed, 0x6e0e_0000 + k as u64);
            let gene = k / 2;
            *byte = match gene {
                COAT => (h % coats.max(1) as u64) as u8,
                PATTERN => (h % patterns.max(1) as u64) as u8,
                MORPH => u8::from(h.is_multiple_of(MORPH_ONE_IN)),
                _ => (h >> 56) as u8,
            };
        }
        Self(bytes)
    }

    /// A genome from a record's bytes. A shorter list (a record written
    /// before a gene existed) takes the default for every gene it lacks; a
    /// longer one (a later build's) keeps the genes this build knows.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut genome = Self::default();
        let whole = bytes.len().min(BYTES) / 2 * 2;
        genome.0[..whole].copy_from_slice(&bytes[..whole]);
        genome
    }

    pub fn alleles(&self, gene: usize) -> (u8, u8) {
        (self.0[2 * gene], self.0[2 * gene + 1])
    }

    /// A quantitative gene's value, 0..1: the mean of its alleles.
    pub fn value(&self, gene: usize) -> f32 {
        let (a, b) = self.alleles(gene);
        (f32::from(a) + f32::from(b)) / 510.0
    }

    /// Body length against the species', 0.85..1.25 (M9's range).
    pub fn size_factor(&self) -> f32 {
        0.85 + 0.40 * self.value(SIZE)
    }

    /// Product amounts against the species', 0.6..1.4.
    pub fn yield_factor(&self) -> f32 {
        0.6 + 0.8 * self.value(YIELD)
    }

    /// How readily it trusts, 1.3 (calm) to 0.6 (wild).
    pub fn trust_factor(&self) -> f32 {
        1.3 - 0.7 * self.value(TEMPERAMENT)
    }

    /// The coat shown: the dominant (earlier) of the two.
    pub fn coat(&self) -> usize {
        let (a, b) = self.alleles(COAT);
        usize::from(a.min(b))
    }

    pub fn pattern(&self) -> usize {
        let (a, b) = self.alleles(PATTERN);
        usize::from(a.min(b))
    }

    /// Whether the rare morph shows: two copies.
    pub fn morph(&self) -> bool {
        self.alleles(MORPH) == (1, 1)
    }
}
