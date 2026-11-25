+++
cruncha-template = 1
id = "mechanical.oRings.radialGland"
version = "1.0.0"
title = "O-ring in a radial gland (piston or rod), Parker"
summary = "An O-ring in a static or reciprocating piston (male) or rod (female) gland: stretch of the ring's ID, its cross-section after stretch, gland depth, squeeze, gland fill and diametral clearance, each at nominal size and at its tolerance extremes, checked against the Parker O-Ring Handbook's limits and the extrusion pressure read off Parker's chart."
category = ["Mechanical engineering", "Seals"]
tags = ["O-ring", "gland", "groove", "seal", "squeeze", "gland fill", "stretch", "extrusion", "clearance", "piston seal", "rod seal", "AS568", "Parker", "elastomer"]
from = "lacewing:mechanical.oRings"
license = "GPL-3.0-or-later"
inputs = ["side", "motion", "ID", "t_ID", "W", "t_W", "D_o", "t_o", "D_i", "t_i", "D_m", "t_m", "G", "t_G", "p", "p_x"]
outputs = ["s", "s_min", "s_max", "W_s", "L", "L_min", "L_max", "δ", "δ_min", "δ_p", "δ_(p,max)", "φ_g", "φ_(g,min)", "φ_(g,max)", "E", "E_max"]

[[references]]
work = "Parker O-Ring Handbook, ORD 5700"
edition = "2018"
locator = "Secs. 3.5 to 3.7 (stretch, squeeze, gland fill: the limits); Figure 3-3 and its empirical formulas (loss of cross-section with stretch); Figure 3-2 (extrusion limits); Design Charts 4-2 (static radial glands) and 5-2-a (reciprocating glands); Table 9-1 (AS568 sizes)"

[[references]]
work = "SAE AS568, Aerospace Size Standard for O-rings"
edition = "AS568F"
locator = "ring inside diameters and cross-sections with their tolerances"

[[tests]]
name = "Parker's static piston gland for 2-214 (Design Table 4-2): stretched past 5 % at the extremes"
source = "Lacewing Lab o_rings_test: ring 0.984 ± 0.010 × 0.139 ± 0.004 in (Parker Table 9-1); bore 1.250 +0.002, groove 1.028 −0.002, plug 1.247 −0.001, width 0.187 +0.005 in, taken as mid-band ± half-band. Gland depth 0.111 to 0.113 in and clearance 0.003 to 0.006 in are Chart 4-2's ranges; stretch 4.37 % nominal but 5.54 % with the ring's ID smallest on the largest groove, over Parker's 5 %. Nominal squeeze 16.889 % and fill 67.202 % are the Lacewing engine's. 1000 psi against 1188 psi, Parker Fig. 3-2's 70 Shore A curve at 0.006 in as the Lacewing engine reads it"
inputs = { side = "1", motion = "1", ID = "0.984 in", t_ID = "0.010 in", W = "0.139 in", t_W = "0.004 in", D_o = "1.251 in", t_o = "0.001 in", D_i = "1.027 in", t_i = "0.001 in", D_m = "1.2465 in", t_m = "0.0005 in", G = "0.1895 in", t_G = "0.0025 in", p = "1000 psi", p_x = "1188 psi" }
expect = { s = "4.36991870", s_max = "5.54414784", L = "0.112 in", L_min = "0.111 in", L_max = "0.113 in", "δ_p" = "16.8892419", "φ_g" = "67.2023101", E = "0.0045 in", E_max = "0.006 in" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Parker's static rod gland for 2-214 (Design Table 4-2): within every limit"
source = "Lacewing Lab o_rings_test: ring as above; rod 1.000 −0.002, housing groove 1.222 +0.002, throat 1.003 +0.001, width 0.187 +0.005 in, mid-band ± half-band. Nominal stretch 1.5244 %, cross-section after stretch 0.137044 in, squeeze 18.2744 %, fill 69.4997 %, clearance 0.0045 in (0.003 to 0.006): the Lacewing engine's values. 1000 psi against 1188 psi from Parker Fig. 3-2 (70 Shore A at 0.006 in)"
inputs = { side = "2", motion = "1", ID = "0.984 in", t_ID = "0.010 in", W = "0.139 in", t_W = "0.004 in", D_o = "1.223 in", t_o = "0.001 in", D_i = "0.999 in", t_i = "0.001 in", D_m = "1.0035 in", t_m = "0.0005 in", G = "0.1895 in", t_G = "0.0025 in", p = "1000 psi", p_x = "1188 psi" }
expect = { s = "1.52439024", W_s = "0.137044040 in", "δ_p" = "18.2744465", "φ_g" = "69.4997029", E = "0.0045 in", E_max = "0.006 in", L_max = "0.113 in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The rod gland at 1500 psi: past the extrusion limit"
source = "By hand: the same gland; 1500 psi is over the 1188 psi at which a 70 Shore A ring starts to extrude through 0.006 in (Parker Fig. 3-2, as the Lacewing engine reads it)"
inputs = { side = "2", motion = "1", ID = "0.984 in", t_ID = "0.010 in", W = "0.139 in", t_W = "0.004 in", D_o = "1.223 in", t_o = "0.001 in", D_i = "0.999 in", t_i = "0.001 in", D_m = "1.0035 in", t_m = "0.0005 in", G = "0.1895 in", t_G = "0.0025 in", p = "1500 psi", p_x = "1188 psi" }
expect = { E_max = "0.006 in" }
tolerance = "1e-6"
checks = "fail"
+++

## O-ring in a radial gland

An O-ring sealing radially: on a piston, in a groove cut in the piston and squeezed against the cylinder bore, or in a rod gland, in a groove cut in the housing and squeezed against the rod. The checks are the Parker O-Ring Handbook's (Secs. 3.5 to 3.7): the ring's inside diameter stretched by at most 5 %, a squeeze of at least 0.007 in and at most 30 % for a static seal (about 16 % for a reciprocating one, 25 % for the smallest cross-sections), and a gland fill of 60 to 85 %, leaving room for swell and thermal expansion. The extrusion limit through the diametral clearance is Parker's chart's.

Each result is given at nominal sizes and at its tolerance extremes. An extreme takes every dimension at its own worst value; where one dimension enters two terms (the groove diameter sets both the stretch and the depth), that is slightly beyond what the part can reach, so the bounds are on the safe side.

For a face seal use the face gland template; for a Quad-Ring, the Quad-Ring template. Rotary O-ring seals, friction and compound compatibility are not covered.

### Gland and service

```calc {description="Gland: 1 piston (male) gland, 2 rod (female) gland"}
side = 1_
```

```calc {description="Service: 1 static, 2 reciprocating"}
motion = 1_
```

### Ring

The ring's inside diameter and cross-section with their ± tolerances, from SAE AS568 (Parker Table 9-1) for a standard size, or the maker's drawing. The defaults are 2-214's.

```calc {description="Inside diameter of the free ring"}
ID = 0.984 in_
```

```calc {description="± tolerance on the ring's inside diameter"}
t_ID = 0.010 in_
```

```calc {description="Cross-section diameter of the ring"}
W = 0.139 in_
```

```calc {description="± tolerance on the cross-section"}
t_W = 0.004 in_
```

### Gland dimensions

Take a two-sided tolerance as its mid-band and half its width (a bore of 1.250 +0.002/−0 in is 1.251 ± 0.001 in). Parker's Design Charts 4-2 (static) and 5-2-a (reciprocating) give the recommended gland depth, clearance and groove width for each cross-section.

```calc {description="Outer diameter: the cylinder bore (piston gland) or the housing groove diameter (rod gland)"}
D_o = 1.251 in_
```

```calc {description="± tolerance on the outer diameter"}
t_o = 0.001 in_
```

```calc {description="Inner diameter the ring's ID sits on: the groove diameter (piston gland) or the rod (rod gland)"}
D_i = 1.027 in_
```

```calc {description="± tolerance on the inner diameter"}
t_i = 0.001 in_
```

```calc {description="Mating diameter across the clearance: the piston OD (piston gland) or the throat bore (rod gland)"}
D_m = 1.2465 in_
```

```calc {description="± tolerance on the mating diameter"}
t_m = 0.0005 in_
```

```calc {description="Groove width"}
G = 0.1895 in_
```

```calc {description="± tolerance on the groove width"}
t_G = 0.0025 in_
```

### Stretch

The ring's inside diameter is stretched over the diameter it sits on (or, in a rod gland, may be slightly compressed: a negative stretch). Stretch thins the cross-section; Parker gives empirical formulas for the observed loss (Figure 3-3), one up to 3 % stretch and one from 3 to 25 %.

```calc
function loss(x)
    return {0 if x ≤ 0; max(0, -0.005 + 1.19 x - 0.19 x^2 - 0.001 x^3 + 0.008 x^4) if x ≤ 3; 0.56 + 0.59 x - 0.0046 x^2 otherwise}
end
```

```calc
> Stretch of the ring's ID, percent
s = 100 (D_i - ID) / ID = ??
> Least stretch
s_min = 100 (D_i - t_i - (ID + t_ID)) / (ID + t_ID) = ??
> Greatest stretch
s_max = 100 (D_i + t_i - (ID - t_ID)) / (ID - t_ID) = ??
> Cross-section after stretch, Parker Fig. 3-3
W_s = W (1 - loss(s) / 100) = ?? in
> Smallest: thinnest ring, most stretch
W_(s,min) = (W - t_W) (1 - loss(s_max) / 100) = ?? in
> Largest: thickest ring, least stretch
W_(s,max) = (W + t_W) (1 - loss(s_min) / 100) = ?? in
```

### Depth, squeeze and fill

```calc
> Gland depth, clearance included
L = (D_o - D_i) / 2 = ?? in
L_min = (D_o - t_o - (D_i + t_i)) / 2 = ?? in
L_max = (D_o + t_o - (D_i - t_i)) / 2 = ?? in
> Squeeze
δ = W_s - L = ?? in
> Least squeeze
δ_min = W_(s,min) - L_max = ?? in
> Squeeze as a percentage of the cross-section
δ_p = 100 δ / W_s = ??
> Greatest squeeze, percent
δ_(p,max) = 100 (W_(s,max) - L_min) / W_(s,max) = ??
> Gland fill: the ring's section over the gland's, percent
φ_g = 100 π W_s^2 / (4 L G) = ??
φ_(g,min) = 100 π W_(s,min)^2 / (4 L_max (G + t_G)) = ??
φ_(g,max) = 100 π W_(s,max)^2 / (4 L_min (G - t_G)) = ??
```

### Clearance and extrusion

The diametral clearance E is the extrusion gap, both sides together, between the grooved part and its mate. Parker's Figure 3-2 gives the pressure at which a ring starts to extrude through it, by hardness (70, 80 and 90 Shore A), without back-up rings; read it at E_max. For silicone and fluorosilicone rings, read the chart at E_max/0.4: Parker reduces the clearance they allow by 60 %. Above the limit, close the clearance, use a harder ring or fit back-up rings.

```calc
> Diametral clearance
E = {D_o - D_m if side == 1; D_m - D_i if side == 2} = ?? in
> Largest clearance
E_max = {D_o + t_o - (D_m - t_m) if side == 1; D_m + t_m - (D_i - t_i) if side == 2} = ?? in
```

```calc {description="Fluid pressure across the seal (gauge)"}
p = 1000 psi_
```

```calc {description="Pressure at which the ring starts to extrude through E_max, Parker Fig. 3-2 for its hardness"}
p_x = 1188 psi_
```

### Checks

```calc {description="Parker's greatest squeeze for the service, percent"}
δ_lim = {30 if motion == 1; 25 if W < 0.08 'in'; 16 otherwise} = ??
```

```calc
@test s_max ≤ 5
```

```calc
@test δ_min ≥ 0.007 'in'
```

```calc
@test δ_(p,max) ≤ δ_lim
```

```calc
@test 60 ≤ φ_(g,min)
```

```calc
@test φ_(g,max) ≤ 85
```

```calc
@test p ≤ p_x
```
