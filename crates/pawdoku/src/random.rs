//! The randomness boundary: the one effect the engine has.
//!
//! A [`RandomStream`] is a stream of draws in `[0, 1)`, begun from a seed, indexed from
//! zero, the same for the same seed and [`RANDOM_VERSION`]. The caller chooses the
//! implementation: [`SeededStream`] is the generator the library ships, and
//! [`ReplayStream`] is the fake a test uses to script its draws. How a draw becomes a
//! decision is the consuming module's arithmetic, not this module's.
//!
//! ```
//! use pawdoku::random::{RandomStream, SeededStream};
//!
//! let mut stream = SeededStream::new(7);
//! let u = stream.next_draw()?;
//! assert!((0.0..1.0).contains(&u));
//! assert_eq!(stream.index(), 1);
//! # Ok::<(), pawdoku::random::RandomError>(())
//! ```

/// The name of the generator the library ships, [`SeededStream`].
///
/// It equals the specification's default `config.random_version`, so a consumer that
/// records this name alongside a seed can replay the same draws. A different generator
/// is a different name: the golden test in `tests/random.rs` pins the first draws of
/// seed zero, and whoever changes those bits changes this constant in the same commit.
///
/// ```
/// assert_eq!(pawdoku::random::RANDOM_VERSION, "seeded-stream-1");
/// ```
pub const RANDOM_VERSION: &str = "seeded-stream-1";

/// Why a stream could not produce a draw.
///
/// The library's [`SeededStream`] never fails; only the [`ReplayStream`] fake does. The
/// `Display` text of each variant is stable API: bindings build their exceptions from it.
///
/// ```
/// use pawdoku::random::RandomError;
///
/// let error = RandomError::Exhausted { index: 3 };
/// assert_eq!(error.to_string(), "the replay stream is exhausted at draw 3");
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum RandomError {
    /// A replay stream was asked for a draw past the end of its script.
    #[error("the replay stream is exhausted at draw {index}")]
    Exhausted {
        /// The index of the draw that was asked for and does not exist.
        index: u64,
    },
    /// A replay script holds a value outside `[0, 1)`, which no stream may draw.
    #[error("draw {index} of the replay script is {value}, outside [0, 1)")]
    OutOfRange {
        /// The index of the first offending value in the script.
        index: u64,
        /// The offending value.
        value: f64,
    },
}

/// A stream of draws in `[0, 1)`, begun from a seed and indexed from zero.
///
/// This is the boundary the specification's `ExactReplay` guarantee describes: "a
/// stream of u in \[0,1), begun from the seed, indexed from zero, the same for the same
/// seed and `random_version`. \[...\] a test supplies the draws through the \[...\] fake."
///
/// The trait is dyn-compatible, so a binding can hold a `Box<dyn RandomStream>`. The
/// library's [`SeededStream`] never returns `Err`; the [`ReplayStream`] fake does when
/// its script ends, which a test would rather hear than a panic.
///
/// ```
/// use pawdoku::random::{RandomStream, ReplayStream, SeededStream};
///
/// fn first_draw(stream: &mut dyn RandomStream) -> f64 {
///     stream.next_draw().unwrap_or(0.0)
/// }
///
/// assert!(first_draw(&mut SeededStream::new(1)) < 1.0);
/// assert_eq!(first_draw(&mut ReplayStream::new(vec![0.5])?), 0.5);
/// # Ok::<(), pawdoku::random::RandomError>(())
/// ```
pub trait RandomStream {
    /// Returns the draw at the current index, in `[0, 1)`, and advances the index by one.
    ///
    /// # Errors
    ///
    /// [`RandomError::Exhausted`] when the stream has no draw at the current index. The
    /// library's [`SeededStream`] never returns an error.
    ///
    /// ```
    /// use pawdoku::random::{RandomError, RandomStream, ReplayStream};
    ///
    /// let mut stream = ReplayStream::new(vec![0.25])?;
    /// assert_eq!(stream.next_draw(), Ok(0.25));
    /// assert_eq!(stream.next_draw(), Err(RandomError::Exhausted { index: 1 }));
    /// # Ok::<(), RandomError>(())
    /// ```
    fn next_draw(&mut self) -> Result<f64, RandomError>;

    /// Returns the index the next draw will carry: zero before the first draw.
    ///
    /// ```
    /// use pawdoku::random::{RandomStream, SeededStream};
    ///
    /// let mut stream = SeededStream::new(0);
    /// assert_eq!(stream.index(), 0);
    /// stream.next_draw()?;
    /// assert_eq!(stream.index(), 1);
    /// # Ok::<(), pawdoku::random::RandomError>(())
    /// ```
    fn index(&self) -> u64;
}

const INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const MIX_1: u64 = 0xBF58_476D_1CE4_E5B9;
const MIX_2: u64 = 0x94D0_49BB_1331_11EB;
/// 2^-53, exact, without a cast: `f64::EPSILON` is 2^-52.
const UNIT: f64 = f64::EPSILON / 2.0;

/// The library's generator, named by [`RANDOM_VERSION`]: `SplitMix64`.
///
/// One `u64` of state, a Weyl increment and two multiply-xorshift rounds per output. The
/// top 53 bits of each output, scaled by 2^-53, give an exact `f64` in `[0, 1)`, so the
/// same seed yields the same draws on every IEEE 754 target, WebAssembly included. It is
/// not a cryptographic generator; nothing in the specifications asks for one.
///
/// ```
/// use pawdoku::random::{RandomStream, SeededStream};
///
/// let mut a = SeededStream::new(2024);
/// let mut b = a.clone();
/// assert_eq!(a.next_draw()?.to_bits(), b.next_draw()?.to_bits());
/// # Ok::<(), pawdoku::random::RandomError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct SeededStream {
    state: u64,
    index: u64,
}

impl SeededStream {
    /// Begins the stream from `seed`, at index zero.
    ///
    /// ```
    /// use pawdoku::random::{RandomStream, SeededStream};
    ///
    /// assert_eq!(SeededStream::new(42).index(), 0);
    /// ```
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            state: seed,
            index: 0,
        }
    }
}

impl RandomStream for SeededStream {
    fn next_draw(&mut self) -> Result<f64, RandomError> {
        self.state = self.state.wrapping_add(INCREMENT);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(MIX_1);
        z = (z ^ (z >> 27)).wrapping_mul(MIX_2);
        let bits = z ^ (z >> 31);
        self.index += 1;
        #[expect(
            clippy::cast_precision_loss,
            reason = "53 bits fit f64's mantissa exactly"
        )]
        let mantissa = (bits >> 11) as f64;
        Ok(mantissa * UNIT)
    }

    fn index(&self) -> u64 {
        self.index
    }
}

/// The fake: a stream that replays a script of draws, then reports exhaustion.
///
/// ```
/// use pawdoku::random::{RandomError, RandomStream, ReplayStream};
///
/// let mut stream = ReplayStream::new(vec![0.1, 0.9])?;
/// assert_eq!(stream.next_draw(), Ok(0.1));
/// assert_eq!(stream.next_draw(), Ok(0.9));
/// assert_eq!(stream.next_draw(), Err(RandomError::Exhausted { index: 2 }));
/// # Ok::<(), RandomError>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ReplayScript")
)]
#[non_exhaustive]
pub struct ReplayStream {
    draws: alloc::vec::Vec<f64>,
    index: u64,
}

/// A [`ReplayStream`] as serialised, checked by [`ReplayStream::new`] on the way in, so
/// a deserialised script cannot hold a draw the boundary forbids.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ReplayScript {
    draws: alloc::vec::Vec<f64>,
    index: u64,
}

#[cfg(feature = "serde")]
impl TryFrom<ReplayScript> for ReplayStream {
    type Error = RandomError;

    fn try_from(script: ReplayScript) -> Result<Self, RandomError> {
        let stream = Self::new(script.draws)?;
        Ok(Self {
            index: script.index,
            ..stream
        })
    }
}

impl ReplayStream {
    /// Scripts the stream with `draws`, replayed in order from index zero.
    ///
    /// # Errors
    ///
    /// [`RandomError::OutOfRange`] for the first value outside `[0, 1)`, NaN included,
    /// so a test cannot script a draw the boundary forbids.
    ///
    /// ```
    /// use pawdoku::random::{RandomError, ReplayStream};
    ///
    /// assert!(ReplayStream::new(vec![0.0, 0.5]).is_ok());
    /// assert_eq!(
    ///     ReplayStream::new(vec![0.5, 1.0]),
    ///     Err(RandomError::OutOfRange { index: 1, value: 1.0 }),
    /// );
    /// ```
    pub fn new(draws: alloc::vec::Vec<f64>) -> Result<Self, RandomError> {
        if let Some((index, &value)) = (0_u64..)
            .zip(&draws)
            .find(|&(_, value)| !(0.0..1.0).contains(value))
        {
            return Err(RandomError::OutOfRange { index, value });
        }
        Ok(Self { draws, index: 0 })
    }
}

impl RandomStream for ReplayStream {
    fn next_draw(&mut self) -> Result<f64, RandomError> {
        let draw = usize::try_from(self.index)
            .ok()
            .and_then(|position| self.draws.get(position))
            .copied()
            .ok_or(RandomError::Exhausted { index: self.index })?;
        self.index += 1;
        Ok(draw)
    }

    fn index(&self) -> u64 {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::{RandomError, RandomStream, ReplayStream, SeededStream};
    use alloc::format;
    use alloc::string::ToString;
    use alloc::vec;
    use proptest::prelude::*;

    #[test]
    fn exhausted_text_is_stable() {
        assert_eq!(
            RandomError::Exhausted { index: 7 }.to_string(),
            "the replay stream is exhausted at draw 7"
        );
    }

    #[test]
    fn out_of_range_text_is_stable() {
        assert_eq!(
            RandomError::OutOfRange {
                index: 2,
                value: 1.5
            }
            .to_string(),
            "draw 2 of the replay script is 1.5, outside [0, 1)"
        );
    }

    #[test]
    fn one_is_out_of_range() {
        assert_eq!(
            ReplayStream::new(vec![0.5, 1.0]),
            Err(RandomError::OutOfRange {
                index: 1,
                value: 1.0
            })
        );
    }

    #[test]
    fn negative_zero_is_in_range() {
        let mut stream = ReplayStream::new(vec![-0.0]).unwrap();
        assert_eq!(stream.next_draw().unwrap().to_bits(), (-0.0_f64).to_bits());
    }

    #[test]
    fn nan_is_out_of_range() {
        let error = ReplayStream::new(vec![0.25, f64::NAN]).unwrap_err();
        assert!(matches!(error, RandomError::OutOfRange { index: 1, value } if value.is_nan()));
    }

    #[test]
    fn exhaustion_reports_the_missing_index() {
        let mut stream = ReplayStream::new(vec![0.5]).unwrap();
        assert_eq!(stream.index(), 0);
        assert_eq!(stream.next_draw(), Ok(0.5));
        assert_eq!(stream.next_draw(), Err(RandomError::Exhausted { index: 1 }));
        assert_eq!(stream.index(), 1);
    }

    #[test]
    fn clones_replay_the_same_draws_and_debug_names_the_type() {
        let mut seeded = SeededStream::new(3);
        let mut replay = ReplayStream::new(vec![0.75]).unwrap();
        let error = RandomError::Exhausted { index: 0 };
        let (mut seeded_copy, mut replay_copy) = (seeded.clone(), replay.clone());
        assert_eq!(seeded, seeded_copy);
        assert_eq!(error.clone(), error);
        assert_eq!(
            seeded.next_draw().unwrap().to_bits(),
            seeded_copy.next_draw().unwrap().to_bits()
        );
        assert_eq!(replay.next_draw(), replay_copy.next_draw());
        assert!(format!("{seeded:?}").starts_with("SeededStream"));
        assert!(format!("{replay:?}").starts_with("ReplayStream"));
        assert!(format!("{error:?}").starts_with("Exhausted"));
    }

    /// Feeds a serialised `ReplayStream`, a sequence of `draws` then `index`, to a
    /// deserializer, through serde's own value deserializers: no format crate needed.
    #[cfg(feature = "serde")]
    fn deserialise_replay(
        draws: vec::Vec<f64>,
        index: u64,
    ) -> Result<ReplayStream, serde::de::value::Error> {
        use serde::Deserialize;
        use serde::de::value::{Error, SeqAccessDeserializer, SeqDeserializer};
        use serde::de::{DeserializeSeed, IntoDeserializer, SeqAccess};

        struct Fields(Option<vec::Vec<f64>>, Option<u64>);

        impl<'de> SeqAccess<'de> for Fields {
            type Error = Error;

            fn next_element_seed<T: DeserializeSeed<'de>>(
                &mut self,
                seed: T,
            ) -> Result<Option<T::Value>, Error> {
                if let Some(draws) = self.0.take() {
                    return seed
                        .deserialize(SeqDeserializer::new(draws.into_iter()))
                        .map(Some);
                }
                self.1
                    .take()
                    .map(|index| seed.deserialize(index.into_deserializer()))
                    .transpose()
            }
        }

        ReplayStream::deserialize(SeqAccessDeserializer::new(Fields(Some(draws), Some(index))))
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialising_keeps_a_valid_script_and_its_index() {
        let mut stream = deserialise_replay(vec![0.25, 0.5], 1).unwrap();
        assert_eq!(stream.index(), 1);
        assert_eq!(stream.next_draw(), Ok(0.5));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialising_rejects_a_draw_outside_the_range() {
        assert_eq!(
            deserialise_replay(vec![0.5, 1.0], 0)
                .unwrap_err()
                .to_string(),
            "draw 1 of the replay script is 1, outside [0, 1)"
        );
        assert!(deserialise_replay(vec![f64::NAN], 0).is_err());
    }

    proptest! {
        #[test]
        fn every_seed_draws_in_range(seed in any::<u64>()) {
            let mut stream = SeededStream::new(seed);
            for _ in 0..64 {
                let u = stream.next_draw().unwrap();
                prop_assert!((0.0..1.0).contains(&u));
            }
        }
    }
}
