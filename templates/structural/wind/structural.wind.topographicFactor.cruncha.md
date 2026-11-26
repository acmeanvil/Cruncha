+++
cruncha-template = 1
id = "structural.wind.topographicFactor"
version = "1.1.0"
title = "Wind topographic factor Kzt (hill, ridge, escarpment), ASCE 7-10"
summary = "Topographic factor Kzt = (1 + K1 K2 K3)² for wind speed-up over an isolated 2-D ridge, 2-D escarpment or 3-D hill, by the equations of Figure 26.8-1."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "topographic factor", "Kzt", "K1", "K2", "K3", "hill", "ridge", "escarpment", "speed-up", "Fig. 26.8-1"]
from = "lacewing:structural.windVelocity"
license = "GPL-3.0-or-later"
inputs = ["H", "L_h", "x", "z", "k_1", "γ", "μ"]
outputs = ["L_(h,e)", "K_1", "K_2", "K_3", "K_zt"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 26.8.1 (when speed-up applies), Sec. 26.8.2 Eq. 26.8-1; Fig. 26.8-1 (equations, parameter table, notes 1-2, multiplier tables for exposure C)"

[[references]]
work = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16 (ICC)"
locator = "Design Example 2, topographic effects (2-D escarpment in exposure D; 3-D hill in exposure C); ASCE 7-16 Fig. 26.8-1 has the same equations"

[[tests]]
name = "Wind Design Manual: escarpment in exposure D, H/Lh above 0.5"
source = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16, Design Example 2, escarpment (verified in the PDF): H 266 ft, Lh 320 ft, x 100 ft downwind, z 33 ft, exposure D (K1/(H/Lh) 0.95, γ 2.5, μ 4): K1 = 0.475, K2 = 0.95, K3 = 0.85, Kzt = 1.93. Exact: K2 0.9530, K3 0.8564, Kzt 1.9252"
inputs = { H = "266 ft", L_h = "320 ft", x = "100 ft", z = "33 ft", k_1 = "0.95", "γ" = "2.5", "μ" = "4" }
expect = { K_1 = "0.475", "L_(h,e)" = "532 ft", K_2 = "0.953", K_3 = "0.8564", K_zt = "1.93" }
tolerance = "0.3%"

[[tests]]
name = "Wind Design Manual: 3-D hill in exposure C, at the crest"
source = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16, Design Example 2, hill (verified in the PDF): H 660 ft, Lh 2100 ft, x 0, z 33 ft (K1/(H/Lh) 1.05, γ 4): K1 0.33, K2 1.00, K3 0.94, Kzt 1.72. Exact: K1 0.3300, K3 0.93908, Kzt 1.7158 (the manual rounds K1 and K3 first)"
inputs = { H = "660 ft", L_h = "2100 ft", x = "0 ft", z = "33 ft", k_1 = "1.05", "γ" = "4", "μ" = "1.5" }
expect = { K_1 = "0.33", K_2 = "1", K_3 = "0.94", K_zt = "1.72" }
tolerance = "0.3%"

[[tests]]
name = "2-D escarpment, exposure C: against Figure 26.8-1's multiplier tables"
source = "ASCE 7-10 Fig. 26.8-1 multiplier tables (exposure C, 2-D escarpment; verified in the PDF): K1 0.26 at H/Lh 0.30, K2 0.88 at x/Lh 0.50 downwind, K3 0.78 at z/Lh 0.10. The equations give 0.255, 0.875, 0.7788"
inputs = { H = "30 ft", L_h = "100 ft", x = "50 ft", z = "10 ft", k_1 = "0.85", "γ" = "2.5", "μ" = "4" }
expect = { K_1 = "0.26", K_2 = "0.88", K_3 = "0.78" }
tolerance = "2%"

[[tests]]
name = "2-D escarpment, exposure C: Kzt by hand"
source = "By hand, Fig. 26.8-1 equations: K1 = 0.85 × 0.30 = 0.255, K2 = 1 − 50/(4 × 100) = 0.875, K3 = e^(−2.5 × 10/100) = 0.77880; Kzt = (1 + 0.255 × 0.875 × 0.77880)² = 1.37774. Lacewing Lab asce7_wind_test checks the same K1, K2 and K3"
inputs = { H = "30 ft", L_h = "100 ft", x = "50 ft", z = "10 ft", k_1 = "0.85", "γ" = "2.5", "μ" = "4" }
expect = { K_zt = "1.377736" }
tolerance = "1e-6"

[[tests]]
name = "2-D ridge, exposure C, upwind: against Figure 26.8-1"
source = "ASCE 7-10 Fig. 26.8-1 multiplier tables (2-D ridge; verified in the PDF): K1 0.72 at H/Lh 0.50, K2 0.33 at |x|/Lh 1.0 (all other cases), K3 0.55 at z/Lh 0.20. Equations: 0.725, 0.3333, 0.5488; Kzt = (1 + 0.725 × 0.3333 × 0.5488)² = 1.28285 by hand"
inputs = { H = "50 ft", L_h = "100 ft", x = "-100 ft", z = "20 ft", k_1 = "1.45", "γ" = "3", "μ" = "1.5" }
expect = { K_1 = "0.72", K_2 = "0.33", K_3 = "0.55", K_zt = "1.2828" }
tolerance = "1.1%"

[[tests]]
name = "Steep 3-D hill: H/Lh above 0.5 taken as 0.5 with Lh = 2H"
source = "Fig. 26.8-1 note 2; Lacewing Lab asce7_wind_test (K1 = 1.05 × 0.5 for H/Lh 1.2, exposure C hill). By hand at the crest, ground level: K1 = 0.525, K2 = K3 = 1, Kzt = 1.525² = 2.325625; Lh used = 120 ft"
inputs = { H = "60 ft", L_h = "50 ft", x = "0 ft", z = "0 ft", k_1 = "1.05", "γ" = "4", "μ" = "1.5" }
expect = { K_1 = "0.525", "L_(h,e)" = "120 ft", K_zt = "2.325625" }
tolerance = "1e-9"

[[tests]]
name = "Valley (H negative): no reduction below 1"
source = "Lacewing Lab asce7_wind_test: H ≤ 0 gives Kzt = 1 (the owner's decision: the figure's negative-H valley case would lower Kzt, which is not taken)"
inputs = { H = "-30 ft", L_h = "100 ft", x = "0 ft", z = "0 ft", k_1 = "1.45", "γ" = "3", "μ" = "1.5" }
expect = { K_zt = "1" }
tolerance = "1e-12"
+++

## Wind topographic factor K_zt (ASCE 7-10)

The speed-up of wind over an isolated hill, ridge or escarpment, as the factor K_zt on the velocity pressure (Eq. 26.8-1). Bind the velocity pressure template's K_zt (structural.wind.velocityPressure) to this one, with the same z. K_2 and K_3 come from the equations of Figure 26.8-1. K_1 is the figure's parameter K_1/(H/L_h) times H/L_h; that parameter, γ and μ are read from the figure's parameter table for the shape and exposure.

Speed-up must be included only when all five conditions of Sec. 26.8.1 hold: the feature is isolated and unobstructed upwind, it stands at least twice as high as the upwind terrain within 2 mi, the structure is in its upper half or near the crest, H/L_h ≥ 0.2, and H is at least 15 ft (exposures C and D) or 60 ft (B). This template does not check them: if they don't all hold, K_zt = 1.

Where the figure's table would let a valley (H negative) reduce K_zt below 1, this template keeps K_zt = 1.

### Feature and site

```calc {description="Height of the hill or escarpment relative to the upwind terrain"}
H = 60 ft_
```

```calc {description="Distance upwind of the crest to where the ground is H/2 below it"}
L_h = 200 ft_
```

```calc {description="Distance from the crest to the site (negative upwind, positive downwind)"}
x = 50 ft_
```

```calc {description="Height above local ground at the site"}
z = 30 ft_
```

### Figure 26.8-1 parameters

These three come from the parameter table of Figure 26.8-1, by the shape of the feature (2-D ridge, 2-D escarpment or 3-D axisymmetric hill), the exposure (K_1/(H/L_h) only) and, for μ, whether the site is upwind or downwind of the crest. The defaults are for a 2-D escarpment in exposure C, downwind.

```calc {description="K1/(H/Lh) for the shape and exposure, ASCE 7-10 Fig. 26.8-1 (0.85 is for a 2-D escarpment in exposure C)"}
k_1 = 0.85_
```

```calc {description="Height attenuation factor γ for the shape, ASCE 7-10 Fig. 26.8-1 (2.5 is for a 2-D escarpment)"}
γ = 2.5_
```

```calc {description="Horizontal attenuation factor μ for the shape, upwind or downwind of the crest, ASCE 7-10 Fig. 26.8-1 (4 is for a 2-D escarpment, downwind)"}
μ = 4_
```

### Multipliers

For H/L_h above 0.5, the figure's note 2 takes H/L_h = 0.5 for K_1 and 2H in place of L_h for K_2 and K_3.

```calc {description="Lh used for K2 and K3 (2H where H/Lh > 0.5)"}
L_(h,e) = {2 H if H / L_h > 0.5; L_h otherwise} = ?? ft
```

```calc {description="Shape and steepness factor K1"}
K_1 = k_1 min(H / L_h, 0.5) = ??
```

```calc {description="Reduction with distance from the crest K2"}
K_2 = max(1 - abs(x) / (μ L_(h,e)), 0) = ??
```

```calc {description="Reduction with height above local terrain K3"}
K_3 = exp(-γ z / L_(h,e)) = ??
```

```calc {description="Topographic factor, Eq. 26.8-1 (not below 1)"}
K_zt = {(1 + K_1 K_2 K_3)^2 if H > 0 'ft'; 1 otherwise} = ??
```
