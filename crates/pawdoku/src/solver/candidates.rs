//! The digits a cell may still hold, as a set.

use crate::sudoku::SIDE;

/// A set of digits from 1 to [`SIDE`]: a bit to a digit, and no other bit ever set,
/// which is `CandidatesAreDigits`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Candidates(u16);

impl Candidates {
    /// No digit.
    pub(super) const NONE: Self = Self(0);

    /// Every digit: what `digits_up_to(config.side)` gives.
    pub(super) const ALL: Self = Self(((1 << SIDE) - 1) << 1);

    /// The one digit `digit`, or no digit when it is not one a cell may hold.
    pub(super) fn only(digit: u8) -> Self {
        let bit = 1_u16.checked_shl(u32::from(digit));
        bit.map_or(Self::NONE, |bit| Self(bit & Self::ALL.0))
    }

    /// How many digits the set holds.
    pub(super) const fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub(super) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(super) fn contains(self, digit: u8) -> bool {
        self.overlaps(Self::only(digit))
    }

    /// Whether the two sets share a digit.
    pub(super) const fn overlaps(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Every digit of either set.
    pub(super) const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The digits of this set that `other` does not hold.
    pub(super) const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// The digits of the set, lowest first.
    pub(super) fn digits(self) -> impl DoubleEndedIterator<Item = u8> {
        (1..=SIDE).filter(move |&digit| self.contains(digit))
    }

    /// The set's digit when it holds exactly one, and nothing otherwise.
    pub(super) fn single(self) -> Option<u8> {
        self.digits().next().filter(|_| self.count() == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::Candidates;
    use alloc::vec::Vec;

    fn of(digits: &[u8]) -> Candidates {
        let only = digits.iter().map(|&digit| Candidates::only(digit));
        only.fold(Candidates::NONE, Candidates::with)
    }

    #[test]
    fn every_digit_is_one_to_nine_each_once() {
        let digits: Vec<u8> = Candidates::ALL.digits().collect();
        assert_eq!(digits, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(Candidates::ALL.count(), 9);
        assert!(!Candidates::ALL.is_empty());
        assert!(Candidates::NONE.is_empty());
        assert_eq!(Candidates::NONE.count(), 0);
    }

    #[test]
    fn candidates_are_digits_whatever_is_offered() {
        for digit in [0, 10, 15, 16, 200, u8::MAX] {
            assert_eq!(Candidates::only(digit), Candidates::NONE);
            assert!(!Candidates::ALL.contains(digit));
        }
        for digit in 1..=9 {
            assert_eq!(Candidates::only(digit).single(), Some(digit));
            assert!(Candidates::ALL.contains(digit));
        }
    }

    #[test]
    fn sets_join_part_and_overlap() {
        let (low, high) = (of(&[1, 2, 3]), of(&[3, 4]));
        assert_eq!(low.with(high), of(&[1, 2, 3, 4]));
        assert_eq!(low.without(high), of(&[1, 2]));
        assert!(low.overlaps(high));
        assert!(!low.overlaps(of(&[4, 9])));
        assert_eq!(low.digits().rev().collect::<Vec<u8>>(), [3, 2, 1]);
    }

    #[test]
    fn a_single_is_a_set_of_exactly_one_digit() {
        assert_eq!(of(&[7]).single(), Some(7));
        assert_eq!(of(&[7, 8]).single(), None);
        assert_eq!(Candidates::NONE.single(), None);
    }
}
