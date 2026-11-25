+++
cruncha-template = 1
id = "mechanical.rollingBearings.dutyCycle"
version = "1.0.0"
title = "Rolling bearing life under a duty cycle (Shigley)"
summary = "A rolling bearing run through a cycle of steps, each with its share of the time, speed, equivalent load and application factor: each step's share of the revolutions, the steady equivalent load that does the same fatigue damage, the mean speed, and the rating life L10 in revolutions and hours."
category = ["Mechanical engineering", "Bearings"]
tags = ["rolling bearing", "ball bearing", "roller bearing", "bearing life", "L10", "duty cycle", "load spectrum", "variable load", "equivalent load", "Miner's rule", "Shigley", "Juvinall"]
from = "lacewing:mechanical.rollingBearings"
license = "GPL-3.0-or-later"
inputs = ["kind", "C_10", "L_R", "t", "n", "F_e", "a_f"]
outputs = ["p", "f", "F_eq", "n_m", "L_10", "L_(10,h)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 11-8 Variable Loading: Eqs. 11-13 to 11-15; Example 11-5; Eq. 11-1 (life exponents)"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 14.7: Sample Problem 14.3 and Problem 14.24 (exponent 10/3, ratings at 90 × 10⁶ revolutions, Table 14.2)"

[[tests]]
name = "Shigley Ex. 11-5: four steps of a ball bearing's duty cycle"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Example 11-5: time shares 0.1, 0.1, 0.3, 0.5 at 2000, 3000, 3000, 2400 rev/min, equivalent loads 794, 626, 878, 668 lbf with application factors 1.10, 1.25, 1.10, 1.25: revolution shares 0.077, 0.115, 0.346, 0.462 and Fe = 882.4 lbf with them exact (the book's 884 lbf rounds the shares to three places), mean speed 2600 rev/min. L10 at a rating of 8000 lbf by hand: (8000/882.40454)³ × 10⁶ = 745.18957e6 rev, 4776.8562 h"
inputs = { kind = "1", C_10 = "8000 lbf", L_R = "1e6", t = "[0.1; 0.1; 0.3; 0.5]", n = "[2000; 3000; 3000; 2400] rpm", F_e = "[794; 626; 878; 668] lbf", a_f = "[1.10; 1.25; 1.10; 1.25]" }
expect = { F_eq = "882.40454 lbf", n_m = "2600 rpm", L_10 = "745.18957e6", "L_(10,h)" = "4776.8562 h" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Juvinall Problem 14.24: No. 312 bearing, 7, 14 and 18 kN at 1800, 1200 and 800 rev/min"
source = "Lacewing Lab rolling_bearings_test, after Juvinall 5th Problem 14.24 (answer about 6400 h): No. 312 radial ball bearing rated 20 kN at 90 × 10⁶ rev (Table 14.2), 55, 25 and 20 % of the time, Juvinall's exponent 10/3 for ball bearings: Fe = 11.569773 kN, mean speed 1450 rev/min, 6413 h"
inputs = { kind = "2", C_10 = "20 kN", L_R = "90e6", t = "[0.55; 0.25; 0.20]", n = "[1800; 1200; 800] rpm", F_e = "[7; 14; 18] kN", a_f = "[1; 1; 1]" }
expect = { F_eq = "11.569773 kN", n_m = "1450 rpm", "L_(10,h)" = "6400 h" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 14.3: No. 207 bearing, 3, 5 and 7 kN for 50, 30 and 20 % at 1000 rev/min"
source = "Lacewing Lab rolling_bearings_test, after Juvinall 5th Sample Problem 14.3: No. 207 rated 8.5 kN at 90 × 10⁶ rev, B10 life 8741 h with the book's exponent 3.33 (8751 h with 10/3)"
inputs = { kind = "2", C_10 = "8.5 kN", L_R = "90e6", t = "[0.5; 0.3; 0.2]", n = "[1000; 1000; 1000] rpm", F_e = "[3; 5; 7] kN", a_f = "[1; 1; 1]" }
expect = { "L_(10,h)" = "8741 h" }
tolerance = "0.2%"
checks = "pass"
+++

## Rolling bearing life under a duty cycle

A bearing whose load and speed change through a repeating cycle, after Shigley Sec. 11-8. Each step i runs for a share t_i of the time at speed n_i under its equivalent load F_e,i (from the bearing's X and Y, as in the rolling bearing life template) with its own application factor. Fatigue damage adds by the revolutions run at each load (the load-life relation F L^(1/p) = constant, equivalent to Miner's rule), so the cycle is replaced by the one steady load F_eq that does the same damage over the same revolutions (Eqs. 11-13 to 11-15). The life in hours is counted at the cycle's mean speed.

### Bearing

The life exponent p is 3 for ball and 10/3 for roller bearings (Shigley Eq. 11-1). Juvinall (Sec. 14.7) follows the makers who use 10/3 for ball bearings too: enter kind 2 for his method. C_10 is the catalogue's rating at L_R revolutions (10⁶ for most makers, 90 × 10⁶ for Timken and Juvinall's Table 14.2).

```calc {description="Life exponent: 1 ball (p = 3), 2 roller (p = 10/3)"}
kind = 1_
```

```calc {description="Basic dynamic load rating C10, from the catalogue"}
C_10 = 8000 lbf_
```

```calc {description="Rating life the catalogue's C10 is for, in revolutions"}
L_R = 1e6_
```

```calc {description="Life exponent, Eq. 11-1"}
p = {3 if kind == 1; 10 / 3 if kind == 2} = ??
```

### Steps of the cycle

Enter one entry per step, as column vectors of the same length. The time shares need not add to 1.

```calc {description="Share of the time spent in each step"}
t = [0.1; 0.1; 0.3; 0.5]_
```

```calc {description="Speed in each step"}
n = [2000; 3000; 3000; 2400] rpm_
```

```calc {description="Equivalent bearing load in each step, Fe = X·V·Fr + Y·Fa"}
F_e = [794; 626; 878; 668] lbf_
```

```calc {description="Application factor of each step, Shigley Table 11-5"}
a_f = [1.10; 1.25; 1.10; 1.25]_
```

### Equivalent load and life

Cruncha's rpm is an angular speed (2π radians a revolution), so the revolutions are counted as n/(2π).

```calc
function dutyLoad(t, n, F, a, p)
    r = sum(t ∘ n)
    F_1 = max(a ∘ F)
    s = 0
    for k in 1:length(t)
        s = s + t[k] n[k] / r (a[k] F[k] / F_1)^p
    end
    return F_1 s^(1 / p)
end
```

```calc
> Share of the revolutions in each step, Eq. 11-14
f = (t ∘ n) / sum(t ∘ n) = ??
> Steady load doing the cycle's damage, Eq. 11-15
F_eq = dutyLoad(t, n, F_e, a_f, p) = ?? lbf
> Mean speed of the cycle
n_m = sum(t ∘ n) / sum(t) = ?? rpm
> Rating life in revolutions, Eq. 11-3
L_10 = L_R (C_10 / F_eq)^p = ??
> Rating life in hours at the mean speed
L_(10,h) = L_10 / (n_m / (2 π)) = ?? h
```
