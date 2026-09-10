//! The 16-byte drill key.
//!
//! Every ore fact, every melted row and every residual is addressed at a
//! [`Facet`]. The address is what makes the slag actionable: a proposer attaches
//! a new convention row at the exact coordinate where the melt failed, rather
//! than widening a match arm somewhere else.
//!
//! # Shape: the V3 content-blind 4 + 12 facet
//!
//! `classid(4) + 12-byte payload`, the payload carved as **six `(u8:u8)`
//! rails** — `6 × 2 = 12`. The `u8:u8` pairs stay two separate bytes and are
//! never widened to a `u16` or a `u24`: a flat `u24` tail carries no axis and
//! cannot hold a rail, which is why the V1 tail shape is not used for new
//! units.
//!
//! ```text
//!   classid u32 | rail0 tu | rail1 fn | rail2 scope | rail3 depth:kind | rail4 seq | rail5 sym
//! ```
//!
//! # `classid` is 0, and that is the sanctioned value
//!
//! The zero-fallback ladder reads a zero classid as *default class, no prefix
//! routing (dormant)*, which leaves the rails alone as the discriminator. That
//! is exactly true of this corpus: OpenGGS is plain C, no concept has been
//! minted for "the function `PC_Run`", and minting one to fill the field would
//! be inventing vocabulary to satisfy a struct. The field keeps its offset, so
//! a later non-zero mint wakes prefix routing with no layout change.

/// One `(u8:u8)` rail. Two bytes, never one `u16`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rail {
    pub hi: u8,
    pub lo: u8,
}

impl Rail {
    /// Split a value across the rail's two bytes.
    ///
    /// Returns `None` above `u16::MAX` rather than truncating: a silently
    /// wrapped index would alias two different functions onto one address, and
    /// an aliased address is worse than a named residual.
    #[must_use]
    pub fn split(v: u32) -> Option<Self> {
        let v = u16::try_from(v).ok()?;
        Some(Self {
            hi: (v >> 8) as u8,
            lo: (v & 0xFF) as u8,
        })
    }

    #[must_use]
    pub const fn value(self) -> u16 {
        ((self.hi as u16) << 8) | self.lo as u16
    }

    #[must_use]
    pub const fn pair(hi: u8, lo: u8) -> Self {
        Self { hi, lo }
    }
}

/// Which rail a value failed to fit into. Named so a residual says *where*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RailId {
    /// rail0 — translation unit index.
    Unit,
    /// rail1 — function index within the corpus.
    Function,
    /// rail2 — scope id within the function.
    Scope,
    /// rail4 — event sequence number within the function.
    Sequence,
    /// rail5 — interned symbol id.
    Symbol,
}

impl RailId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Function => "function",
            Self::Scope => "scope",
            Self::Sequence => "sequence",
            Self::Symbol => "symbol",
        }
    }
}

/// The 16-byte address.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Facet {
    pub classid: u32,
    /// rail0 translation unit · rail1 function · rail2 scope ·
    /// rail3 `depth:kind` · rail4 sequence · rail5 symbol.
    pub rails: [Rail; 6],
}

/// The canonical 16-byte width, asserted rather than described.
const _: () = assert!(size_of::<Facet>() == 16);

impl Facet {
    #[must_use]
    pub const fn unit(&self) -> u16 {
        self.rails[0].value()
    }
    #[must_use]
    pub const fn function(&self) -> u16 {
        self.rails[1].value()
    }
    #[must_use]
    pub const fn scope(&self) -> u16 {
        self.rails[2].value()
    }
    /// rail3 is `depth : kind` — the one rail whose two bytes carry different
    /// quantities, which is what an axis-grouped register is for.
    #[must_use]
    pub const fn depth(&self) -> u8 {
        self.rails[3].hi
    }
    #[must_use]
    pub const fn kind_byte(&self) -> u8 {
        self.rails[3].lo
    }
    #[must_use]
    pub const fn sequence(&self) -> u16 {
        self.rails[4].value()
    }
    #[must_use]
    pub const fn symbol(&self) -> u16 {
        self.rails[5].value()
    }

    /// The address prefix `depth` rails deep, for longest-prefix-wins lookup.
    ///
    /// Depth 0 is the classid alone (the corpus-wide default row); depth 6 is
    /// the fully-qualified event.
    #[must_use]
    pub fn prefix(&self, depth: usize) -> Self {
        let mut out = Self {
            classid: self.classid,
            rails: [Rail::default(); 6],
        };
        for i in 0..depth.min(6) {
            out.rails[i] = self.rails[i];
        }
        out
    }

    /// Little-endian bytes: classid first, then the rails in order, `hi` before
    /// `lo`. This IS the wire form — there is no separate serialiser.
    #[must_use]
    pub fn to_le_bytes(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(&self.classid.to_le_bytes());
        for (i, r) in self.rails.iter().enumerate() {
            b[4 + i * 2] = r.hi;
            b[5 + i * 2] = r.lo;
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rail_round_trips_and_refuses_to_truncate() {
        assert_eq!(Rail::split(0xBEEF).map(Rail::value), Some(0xBEEF));
        // The falsifier for silent aliasing: 0x1_0000 must NOT come back as 0.
        assert_eq!(Rail::split(0x1_0000), None);
        assert_eq!(
            Rail::split(u32::from(u16::MAX)).map(Rail::value),
            Some(u16::MAX)
        );
    }

    #[test]
    fn rails_stay_two_bytes_and_the_facet_stays_sixteen() {
        let f = Facet {
            classid: 0,
            rails: [
                Rail::pair(1, 2),
                Rail::pair(3, 4),
                Rail::pair(5, 6),
                Rail::pair(7, 8),
                Rail::pair(9, 10),
                Rail::pair(11, 12),
            ],
        };
        let b = f.to_le_bytes();
        assert_eq!(b.len(), 16);
        assert_eq!(
            &b[4..8],
            &[1, 2, 3, 4],
            "rail bytes are laid down hi then lo, in rail order"
        );
    }

    #[test]
    fn prefix_zeroes_only_the_rails_below_the_depth() {
        let f = Facet {
            classid: 7,
            rails: [
                Rail::pair(1, 1),
                Rail::pair(2, 2),
                Rail::pair(3, 3),
                Rail::pair(4, 4),
                Rail::pair(5, 5),
                Rail::pair(6, 6),
            ],
        };
        let p = f.prefix(2);
        assert_eq!(p.classid, 7, "the classid survives every prefix");
        assert_eq!(p.rails[0], Rail::pair(1, 1));
        assert_eq!(p.rails[1], Rail::pair(2, 2));
        assert_eq!(p.rails[2], Rail::default(), "rail 2 is below the depth");
        assert_eq!(f.prefix(6), f, "a full-depth prefix is the facet itself");
        assert_ne!(f.prefix(5), f, "depth 5 must still differ from the whole");
    }

    #[test]
    fn depth_and_kind_share_rail_three_without_colliding() {
        let f = Facet {
            classid: 0,
            rails: [
                Rail::default(),
                Rail::default(),
                Rail::default(),
                Rail::pair(3, 9),
                Rail::default(),
                Rail::default(),
            ],
        };
        assert_eq!(f.depth(), 3);
        assert_eq!(f.kind_byte(), 9);
    }
}
