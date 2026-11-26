+++
cruncha-template = 1
id = "structural.seismic.verticalDistribution"
version = "1.0.0"
title = "Vertical distribution of seismic forces, ASCE 7-10 Sec. 12.8.3"
summary = "The equivalent lateral force procedure's distribution of the base shear over the height: the exponent k from the period, the vertical distribution factors Cvx, the force at each level Fx and the storey shears Vx."
category = ["Structural engineering", "Loads"]
tags = ["ASCE 7", "ASCE 7-10", "seismic", "earthquake", "equivalent lateral force", "ELF", "vertical distribution", "Cvx", "Fx", "storey shear", "story shear"]
from = "lacewing:structural.seismic"
license = "GPL-3.0-or-later"
inputs = ["w_x", "h_x", "V", "T"]
outputs = ["k", "C_vx", "F_x", "V_x", "F_(x,top)", "V_(x,base)", "V_(x,top)"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 12.8.3 (Eqs. 12.8-11, 12.8-12, exponent k), Sec. 12.8.4 (Eq. 12.8-13)"

[[references]]
work = "2018 IBC SEAOC Structural/Seismic Design Manual, Vol. 1 (Code Application Examples; ASCE 7-16)"
edition = "2018"
locator = "Design Example 32 (vertical distribution of seismic forces)"

[[tests]]
name = "Three levels, k = 1"
source = "Lacewing Lab asce7_loads_test: weights 100, 100, 50 at 4, 8, 12 (m) with V = 30 and k = 1 (T ≤ 0.5 s): Σwh = 1800; the top level takes 30 × 600/1800 = 10 and the base storey shear is 30; the top storey shear equals the top force"
inputs = { w_x = "[100; 100; 50] kN", h_x = "[4; 8; 12] m", V = "30 kN", T = "0.4 s" }
expect = { k = "1", "F_(x,top)" = "10 kN", "V_(x,base)" = "30 kN", "V_(x,top)" = "10 kN" }
tolerance = "1e-9"

[[tests]]
name = "Lacewing's opening building: five levels of 1000 kN over 60 ft, T = 0.7408 s"
source = "By hand, Eqs. 12.8-11 to 12.8-13: k = 1 + (0.7408 − 0.5)/2 = 1.1204; with levels at 12, 24, … 60 ft, Σw(h/12 ft)^k = 1000 × (1 + 2.1741 + 3.4242 + 4.7266 + 6.0691) = 17393.9; Cvx at the roof = 6069.1/17393.9 = 0.34892, so Fx = 0.34892 × 506.2 = 176.62 kN"
inputs = { w_x = "[1000; 1000; 1000; 1000; 1000] kN", h_x = "[12; 24; 36; 48; 60] ft", V = "506.2 kN", T = "0.74079 s" }
expect = { k = "1.120395", "F_(x,top)" = "176.62 kN", "V_(x,base)" = "506.2 kN" }
tolerance = "0.05%"

[[tests]]
name = "Long period: k = 2"
source = "By hand: T = 3 s ≥ 2.5 s, so k = 2 (Sec. 12.8.3); two equal levels at 10 and 20 ft share V = 100 kips as 100/500 and 400/500: 20 and 80 kips"
inputs = { w_x = "[500; 500] kips", h_x = "[10; 20] ft", V = "100 kips", T = "3 s" }
expect = { k = "2", "F_(x,top)" = "80 kips", "V_(x,base)" = "100 kips" }
tolerance = "1e-9"
[[tests]]
name = "SEAOC Design Example 32: nine-storey steel moment frame"
source = "2018 IBC SEAOC Structural/Seismic Design Manual Vol. 1 (ASCE 7-16; Sec. 12.8.3 is unchanged from 7-10), Design Example 32, verified against the PDF: W 3762 kips, Cs 0.062, V = 233.2 kips, T 1.06 s, k = 1.28; levels 1 to 9 at 20, 32, … 116 ft weighing 465, 440, 422, 422, 584, 405, 405, 405, 214 kips; Σwh^k = 807,000 and the roof force 27.3 kips (the book rounds wh^k to three figures; unrounded it is 27.19, within 0.5%)"
inputs = { w_x = "[465; 440; 422; 422; 584; 405; 405; 405; 214] kips", h_x = "[20; 32; 44; 56; 68; 80; 92; 104; 116] ft", V = "233.2 kips", T = "1.06 s" }
expect = { k = "1.28", "F_(x,top)" = "27.3 kips", "V_(x,base)" = "233.2 kips", "V_(x,top)" = "27.3 kips" }
tolerance = "0.5%"
+++

## Vertical distribution of seismic forces, ASCE 7-10

Distributes the seismic base shear V of the equivalent lateral force procedure to the levels of the building (Sec. 12.8.3), and sums the forces above each storey to give the storey shears (Sec. 12.8.4). The force at a level is in proportion to its weight times its height raised to the power k; k grows with the period, which moves force up the building to allow for the higher modes.

The seismic base shear template gives V and the period T. The horizontal distribution of each storey shear to the vertical elements, torsion and overturning are separate checks.

### Building

List the levels from the lowest up: the effective seismic weight assigned to each level (Sec. 12.7.2) and its height above the base.

```calc {description="Effective seismic weight at each level, bottom first"}
w_x = [1000; 1000; 1000; 1000; 1000] kN_
```

```calc {description="Height of each level above the base, bottom first"}
h_x = [12; 24; 36; 48; 60] ft_
```

```calc {description="Seismic base shear, Eq. 12.8-1"}
V = 506.2 kN_
```

```calc {description="Fundamental period used for Cs"}
T = 0.7408 s_
```

### Distribution

k is 1 for periods of 0.5 s or less, 2 for 2.5 s or more, and interpolated linearly between (Sec. 12.8.3 also allows 2 throughout).

```calc {description="Distribution exponent k, Sec. 12.8.3"}
k = {1 if T ≤ 0.5 's'; 2 if T ≥ 2.5 's'; 1 + (T - 0.5 's') / (2 's') otherwise} = ??
```

Heights enter as a ratio to the lowest level's, which leaves C_vx unchanged and keeps the units whole.

```calc
> Weight times height to the power k, for each level
wh_x = w_x ∘ (h_x / h_x[1])^k = ?? kN
> Vertical distribution factor, Eq. 12.8-12
C_vx = wh_x / sum(wh_x) = ??
> Lateral force at each level, Eq. 12.8-11
F_x = C_vx V = ?? kN
```

The storey shear below level x is the sum of the forces at x and above (Eq. 12.8-13). The function sums the forces as fractions of V, since a matrix built inside a function can't carry a unit.

```calc
function storeyShear(F)
    n = length(F)
    s = F[n]
    v = [s]
    for j in 2:n
        s = s + F[n + 1 - j]
        v = vcat(s, v)
    end
    return v
end
```

```calc {description="Storey shear below each level, Eq. 12.8-13, bottom first"}
V_x = storeyShear(F_x / V) V = ?? kN
```

```calc {description="Force at the top level"}
F_(x,top) = F_x[length(F_x)] = ?? kN
```

```calc {description="Storey shear in the lowest storey (equals V)"}
V_(x,base) = V_x[1] = ?? kN
```

```calc {description="Storey shear in the top storey (equals the top force)"}
V_(x,top) = V_x[length(V_x)] = ?? kN
```
