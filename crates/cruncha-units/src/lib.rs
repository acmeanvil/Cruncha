// SPDX-License-Identifier: GPL-3.0-or-later

//! Cruncha units: Hurmet's unit database and dimension algebra, plus the exact rational numbers
//! that unit factors are written in.
//!
//! This is a port of Hurmet's `units.js` and `rational.js` (spike S2b, see
//! `docs/spikes/S2b-calc-evaluator.md`). It is used by `cruncha-eval`, and it is usable on its own:
//!
//! ```
//! use cruncha_units::{unit_from_name, Currencies};
//! let u = unit_from_name("kip·ft", &Currencies::new()).unwrap();
//! assert_eq!(u.expos, [2.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // force × length
//! assert_eq!(u.factor.to_f64(), 4448.2216152605 * 0.3048);
//! ```
//!
//! - [`Rational`]: Hurmet's `Rnl`, big-integer fractions, deliberately not auto-reduced.
//! - [`unit_from_name`]: `kip·ft`, `lbf/ft`, `N·m/s`, `in²`, `m^2`, `°`, `%`, SI prefixes,
//!   plurals, gauge units such as `°F`, and currencies through a caller-supplied
//!   [`Currencies`] table.
//! - [`jsnum`]: JavaScript number formatting and parsing, which Hurmet's results depend on.

pub mod jsmath;
pub mod jsnum;
pub mod rational;
mod table;
mod units;

pub use rational::{NUMBER_PATTERN, Rational};
pub use units::{
    ALL_ZEROS, Currencies, Dims, Unit, UnitError, is_known_unit, unit_from_name,
    units_are_compatible,
};
