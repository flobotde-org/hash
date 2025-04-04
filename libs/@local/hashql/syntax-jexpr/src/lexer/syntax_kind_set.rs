use core::fmt::{self, Display};

use super::syntax_kind::SyntaxKind;

pub(crate) struct SyntaxKindSetIter(u128);

impl Iterator for SyntaxKindSetIter {
    type Item = SyntaxKind;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let kind = self.0.trailing_zeros();
        self.0 &= !(1 << kind);
        Some(match kind {
            0 => SyntaxKind::String,
            1 => SyntaxKind::Number,
            2 => SyntaxKind::True,
            3 => SyntaxKind::False,
            4 => SyntaxKind::Null,
            5 => SyntaxKind::Comma,
            6 => SyntaxKind::Colon,
            7 => SyntaxKind::LBrace,
            8 => SyntaxKind::RBrace,
            9 => SyntaxKind::LBracket,
            10 => SyntaxKind::RBracket,
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SyntaxKindSet(u128);

impl SyntaxKindSet {
    pub(crate) const EMPTY: Self = Self(0);

    pub(crate) const fn from_slice(slice: &[SyntaxKind]) -> Self {
        let mut set = 0;

        // cannot use for loop here, because it's not const
        let mut index = 0;
        while index < slice.len() {
            set |= slice[index].into_u128();
            index += 1;
        }

        Self(set)
    }

    #[must_use]
    pub(crate) const fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        self.0 & kind.into_u128() != 0
    }
}

impl FromIterator<SyntaxKind> for SyntaxKindSet {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SyntaxKind>,
    {
        let mut set = 0;
        for kind in iter {
            set |= kind.into_u128();
        }
        Self(set)
    }
}

impl IntoIterator for SyntaxKindSet {
    type IntoIter = SyntaxKindSetIter;
    type Item = SyntaxKind;

    fn into_iter(self) -> Self::IntoIter {
        SyntaxKindSetIter(self.0)
    }
}

impl Display for SyntaxKindSet {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.len();
        if count == 0 {
            return fmt.write_str("none");
        }

        for (index, kind) in self.into_iter().enumerate() {
            if index > 0 {
                fmt.write_str(", ")?;
            }

            Display::fmt(&kind, fmt)?;
        }

        Ok(())
    }
}
