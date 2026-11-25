+++
cruncha-template = 1
id = "mechanical.toleranceStack.worstCaseRss"
version = "1.0.0"
title = "One-dimensional tolerance stack, worst case and RSS"
summary = "A chain of dimensions around a loop, each adding to or taking from the result, with equal or unequal tolerances: the mean result, its worst-case and root-sum-square (statistical) tolerances and limits, each dimension's share of each, and the worst-case range checked against the least and greatest result allowed."
category = ["Mechanical engineering", "GD&T"]
tags = ["tolerance stack", "stack-up", "tolerance chain", "worst case", "RSS", "root sum square", "statistical tolerancing", "gap", "clearance", "assembly", "Shigley"]
from = "lacewing:mechanical.toleranceStack"
license = "GPL-3.0-or-later"
inputs = ["L", "s", "t_u", "t_l", "w_min", "w_max"]
outputs = ["L_m", "t", "w_0", "w", "t_wc", "w_(wc,lo)", "w_(wc,hi)", "t_rss", "w_(rss,lo)", "w_(rss,hi)", "f_wc", "f_rss"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 1-14, Dimensions and Tolerances (absolute and statistical tolerance systems); Example 1-7"

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "3.60 and 5.18 (statistical tolerancing), 3.62 (tolerance as the difference between the limits)"

[[tests]]
name = "Shigley Ex. 1-7(a): shoulder screw and three sleeves"
source = "Lacewing Lab stack_up_test, after Shigley 10th Example 1-7: a = 1.750 ± 0.003, b = 0.750 ± 0.001, c = 0.120 ± 0.005, d = 0.875 ± 0.001 in give w = 0.005 ± 0.010 in, −0.005 to 0.015 in; RSS √(9 + 1 + 25 + 1) thousandths = 0.006 in"
inputs = { L = "[1.750; 0.750; 0.120; 0.875] in", s = "[1; -1; -1; -1]", t_u = "[0.003; 0.001; 0.005; 0.001] in", t_l = "[0.003; 0.001; 0.005; 0.001] in", w_min = "-0.01 in", w_max = "0.02 in" }
expect = { w = "0.005 in", w_0 = "0.005 in", t_wc = "0.010 in", "w_(wc,lo)" = "-0.005 in", "w_(wc,hi)" = "0.015 in", t_rss = "0.006 in", "w_(rss,lo)" = "-0.001 in", "w_(rss,hi)" = "0.011 in" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Ex. 1-7(b) against a greatest gap of 0.020 in: the gap can be too wide"
source = "Shigley 10th Example 1-7: with d = 0.867 in, w = 0.013 ± 0.010 in, so the gap runs from 0.003 to 0.023 in, over an upper limit of 0.020 in"
inputs = { L = "[1.750; 0.750; 0.120; 0.867] in", s = "[1; -1; -1; -1]", t_u = "[0.003; 0.001; 0.005; 0.001] in", t_l = "[0.003; 0.001; 0.005; 0.001] in", w_min = "0.003 in", w_max = "0.02 in" }
expect = { w = "0.013 in", "w_(wc,lo)" = "0.003 in", "w_(wc,hi)" = "0.023 in" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Shigley Ex. 1-7(b): a 0.867 in sleeve d keeps the gap at least 0.003 in"
source = "Lacewing Lab stack_up_test, after Shigley 10th Example 1-7: with d = 0.867 in, w = 0.013 ± 0.010 in, so the least gap is 0.003 in; no upper limit (set out of the way at 1 in)"
inputs = { L = "[1.750; 0.750; 0.120; 0.867] in", s = "[1; -1; -1; -1]", t_u = "[0.003; 0.001; 0.005; 0.001] in", t_l = "[0.003; 0.001; 0.005; 0.001] in", w_min = "0.003 in", w_max = "1 in" }
expect = { "w_(wc,lo)" = "0.003 in", t_rss = "0.006 in" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Ex. 1-7(a) against a gap of at least 0.003 in: interference is possible"
source = "Shigley 10th Example 1-7: the least gap −0.005 in is under 0.003 in"
inputs = { L = "[1.750; 0.750; 0.120; 0.875] in", s = "[1; -1; -1; -1]", t_u = "[0.003; 0.001; 0.005; 0.001] in", t_l = "[0.003; 0.001; 0.005; 0.001] in", w_min = "0.003 in", w_max = "1 in" }
expect = { "w_(wc,lo)" = "-0.005 in" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Unequal tolerance: 30 ± 0.1 less 10 +0.2/0 mm"
source = "Lacewing Lab stack_up_test: nominal result 20, mean 30 − 10.1 = 19.9, worst case ± 0.2 gives 19.7 to 20.1 mm; RSS √(0.1² + 0.1²) = 0.141421 mm"
inputs = { L = "[30; 10] mm", s = "[1; -1]", t_u = "[0.1; 0.2] mm", t_l = "[0.1; 0] mm", w_min = "19.5 mm", w_max = "20.5 mm" }
expect = { w_0 = "20 mm", w = "19.9 mm", t_wc = "0.2 mm", "w_(wc,lo)" = "19.7 mm", "w_(wc,hi)" = "20.1 mm", t_rss = "0.1414214 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## One-dimensional tolerance stack

A chain of dimensions around a loop that closes on one result: a gap, a length or a clearance (Shigley Sec. 1-14). Each dimension adds to the result or takes from it. The stack is one-dimensional: every dimension lies along the same direction, and angles, form errors and geometric tolerances that act along it are not included unless entered as dimensions of their own.

Two systems are worked. The worst case (Shigley's absolute system) puts every dimension at its limit at once, so any set of parts in tolerance falls in its range. Root sum of squares (the statistical system) is narrower: it holds when every dimension is normally distributed and its tolerance is the same number of standard deviations, which Shigley warns is rare; a Monte Carlo simulation is the alternative otherwise. A drawing that relies on statistical tolerances marks them with the statistical tolerance symbol and asks for statistical process control (ASME Y14.5-2018 5.18).

### Dimensions

Enter the dimensions as column vectors of the same length, in the loop's order. Each nominal is positive; its sign is +1 when it adds to the result and −1 when it takes from it. The tolerances above and below the nominal are both positive: 10 +0.2/−0.1 is 0.2 above and 0.1 below, and 10 +0.2/0 is 0.2 above and 0 below. A dimension with only a general tolerance takes the ± of the title block's class (ISO 2768-1 Table 1, for example).

```calc {description="Nominal of each dimension, positive"}
L = [1.750; 0.750; 0.120; 0.875] in_
```

```calc {description="Sign of each dimension: +1 adds to the result, −1 takes from it"}
s = [1; -1; -1; -1]_
```

```calc {description="Tolerance above each nominal (positive)"}
t_u = [0.003; 0.001; 0.005; 0.001] in_
```

```calc {description="Tolerance below each nominal (positive)"}
t_l = [0.003; 0.001; 0.005; 0.001] in_
```

### Each dimension as a mean and an equal tolerance

A dimension with unequal tolerances is restated as the midpoint of its limits with an equal bilateral tolerance, half its total tolerance. Neither source prints this step; it follows from the limits, and it is how Shigley's example stacks equal tolerances.

```calc
> Mean of each dimension, midway between its limits
L_m = L + (t_u - t_l) / 2 = ?? in
> Equal bilateral tolerance of each, half its total tolerance
t = (t_u + t_l) / 2 = ?? in
```

### Result

```calc
> Result at the nominal sizes, as drawn
w_0 = sum(s ∘ L) = ?? in
> Mean result
w = sum(s ∘ L_m) = ?? in
```

```calc
> Worst-case tolerance of the result, the sum of the tolerances
t_wc = sum(t) = ?? in
> Least and greatest result, worst case
w_(wc,lo) = w - t_wc = ?? in
w_(wc,hi) = w + t_wc = ?? in
```

```calc
> Statistical tolerance of the result, root sum of squares
t_rss = sqrt(sum(t^2)) = ?? in
> Least and greatest result, RSS
w_(rss,lo) = w - t_rss = ?? in
w_(rss,hi) = w + t_rss = ?? in
```

Each dimension's share shows where the tolerance comes from: its part of the worst-case sum, and its part of the RSS variance. Tightening the dimensions with the largest shares does the most.

```calc
> Share of each dimension in the worst-case tolerance
f_wc = t / t_wc = ??
> Share of each dimension in the RSS variance
f_rss = t^2 / t_rss^2 = ??
```

### Requirement

The least and greatest result the design allows: a gap of at least w_min, a length of at most w_max. Set a limit the design doesn't have well out of the way. The check is by worst case; where statistical tolerances are accepted, compare the RSS range with the same limits instead.

```calc {description="Least result allowed"}
w_min = 0.003 in_
```

```calc {description="Greatest result allowed"}
w_max = 0.020 in_
```

### Check

```calc
@test w_min ≤ w_(wc,lo)
```

```calc
@test w_(wc,hi) ≤ w_max
```
