//! How a draw chooses among `n` things in a stated order: `index_among`.

/// The index a draw chooses among `n` things, the first of which has index 0: the
/// product of `u` and `n` in binary64, truncated toward zero.
///
/// This is not the whole part of the exact product, and the two differ for a few draws
/// beside a boundary; the specification says which this module means. For a draw in
/// `[0, 1)` and an `n` up to 81 the result is below `n`.
///
/// A stream may break the boundary's word and give a draw that is not in `[0, 1)`.
/// The cast saturates, so a draw below zero or one that is not a number picks the
/// first, and the result is held to the last, so a draw of one or more picks that. No
/// draw makes an index outside the `n` things.
///
/// The casts are the arithmetic the specification states. The product of a draw and
/// `n` is never negative, so truncating it is its floor, and `f64::floor` is not in
/// `core`.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "n is at most 81, exact in f64; truncating the product is its floor"
)]
pub(super) fn index_among(u: f64, n: usize) -> usize {
    ((u * n as f64) as usize).min(n.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::index_among;

    /// The largest draw a stream can give: one less 2^-53.
    const LARGEST_DRAW: f64 = 1.0 - f64::EPSILON / 2.0;

    #[test]
    fn a_draw_of_zero_picks_the_first() {
        for n in 1..=81 {
            assert_eq!(index_among(0.0, n), 0);
        }
    }

    #[test]
    fn the_largest_draw_picks_the_last_and_never_n() {
        for n in 1..=81 {
            assert_eq!(index_among(LARGEST_DRAW, n), n - 1);
        }
    }

    /// The specification's worked case. The exact product of this draw and 3 is
    /// `2 - 2^-53`, whose whole part is 1; the binary64 product rounds to 2.0, and
    /// that reading is the module's. An implementation in exact arithmetic answers 1.
    #[test]
    fn the_index_is_the_binary64_product_truncated() {
        let draw = 6_004_799_503_160_661.0 / 9_007_199_254_740_992.0;
        assert_eq!(index_among(draw, 3), 2);
    }

    /// `ReplayStream` refuses such draws; a stream of a caller's own may give them.
    #[test]
    fn a_draw_the_boundary_forbids_still_picks_one_of_the_n() {
        for n in 1..=81 {
            assert_eq!(index_among(1.0, n), n - 1);
            assert_eq!(index_among(f64::INFINITY, n), n - 1);
            assert_eq!(index_among(-0.5, n), 0);
            assert_eq!(index_among(f64::NAN, n), 0);
        }
        assert_eq!(index_among(0.5, 0), 0);
    }
}
