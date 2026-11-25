+++
cruncha-template = 1
id = "mechanical.journalBearings.lengthRatio"
version = "1.0.0"
title = "Raimondi-Boyd variable at any l/d (Eq. 12-16)"
summary = "Raimondi and Boyd's interpolation formula: a journal bearing performance variable (h0/c, ε, (r/c)f, Q/(rcNl), Qs/Q, P/pmax, an angle) at any length-to-diameter ratio from 1/4 up, from its values on the charts' four curves l/d = ∞, 1, 1/2 and 1/4 at the same Sommerfeld number."
category = ["Mechanical engineering", "Bearings"]
tags = ["journal bearing", "Raimondi", "Boyd", "interpolation", "length to diameter", "l/d", "Sommerfeld number", "hydrodynamic bearing", "Shigley"]
from = "lacewing:mechanical.journalBearings"
license = "GPL-3.0-or-later"
inputs = ["λ", "y_L", "y_1", "y_2", "y_4"]
outputs = ["y"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-8, Eq. 12-16 (Raimondi and Boyd's interpolation between l/d = ∞, 1, 1/2, 1/4)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 13.9, Eq. 13.12"

[[tests]]
name = "Friction variable at l/d = 3/4 and S = 0.2"
source = "Lacewing Lab engine (journal_bearings.dart, raimondiBoyd, which applies Eq. 12-16 to its interpolation of Raimondi and Boyd's table): (r/c)f at S = 0.2 is 4.019676, 4.622321, 5.665920 and 7.171847 for l/d = ∞, 1, 1/2, 1/4, and 4.944700 at l/d = 0.75"
inputs = { "λ" = "0.75", y_L = "4.019676179", y_1 = "4.622321252", y_2 = "5.665919766", y_4 = "7.171846737" }
expect = { y = "4.944700379" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Eccentricity ratio at l/d = 3/4 and S = 0.2"
source = "Lacewing Lab engine as above: ε at S = 0.2 is 0.118473, 0.471498, 0.685235 and 0.825946 for l/d = ∞, 1, 1/2, 1/4, and 0.556602 at l/d = 0.75"
inputs = { "λ" = "0.75", y_L = "0.1184733007", y_1 = "0.4714979999", y_2 = "0.6852350915", y_4 = "0.8259462738" }
expect = { y = "0.5566015816" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Hand check: the formula returns the curve itself at l/d = 1/2"
source = "By hand: at l/d = 1/2 the ∞, 1 and 1/4 coefficients vanish and the 1/2 one is (1/4)(1/2)(1)/(1/2)³ = 1"
inputs = { "λ" = "0.5", y_L = "1", y_1 = "2", y_2 = "3", y_4 = "4" }
expect = { y = "3" }
checks = "pass"

[[tests]]
name = "Outside the formula's range: l/d = 0.2"
source = "Shigley Sec. 12-8: Eq. 12-16 covers l/d from 1/4 up. By hand, at 0.2 the coefficients are −0.012, 0.04, −0.04 and 0.02, over 0.008: y = (−0.012 + 0.08 − 0.12 + 0.08)/0.008 = 3.5"
inputs = { "λ" = "0.2", y_L = "1", y_1 = "2", y_2 = "3", y_4 = "4" }
expect = { y = "3.5" }
tolerance = "1e-9"
checks = "fail"
+++

## Raimondi and Boyd's variables at any l/d

Raimondi and Boyd computed their bearings for l/d = ∞ (the long bearing), 1, 1/2 and 1/4, and the charts show those curves only. For another ratio, read the variable off all four curves at the bearing's Sommerfeld number, and combine them with their interpolation formula (Shigley Eq. 12-16, Juvinall Eq. 13.12). It applies to every variable of the charts in turn. The formula is a cubic through the four ratios in 1/(l/d), and holds for l/d from 1/4 up.

```calc {description="Length-to-diameter ratio l/d of the bearing"}
λ = 0.75_
```

```calc {description="The variable on the l/d = ∞ curve (the long bearing), at the bearing's S"}
y_L = 4.019676_
```

```calc {description="The variable on the l/d = 1 curve, at the bearing's S"}
y_1 = 4.622321_
```

```calc {description="The variable on the l/d = 1/2 curve, at the bearing's S"}
y_2 = 5.665920_
```

```calc {description="The variable on the l/d = 1/4 curve, at the bearing's S"}
y_4 = 7.171847_
```

```calc {description="The variable at l/d = λ, Shigley Eq. 12-16"}
y = (-(1 - λ) (1 - 2 λ) (1 - 4 λ) y_L / 8 + (1 - 2 λ) (1 - 4 λ) y_1 / 3 - (1 - λ) (1 - 4 λ) y_2 / 4 + (1 - λ) (1 - 2 λ) y_4 / 24) / λ^3 = ??
```

```calc
@test λ ≥ 0.25
```
