+++
cruncha-template = 1
id = "mechanical.quadRings.radialGland"
version = "1.0.0"
title = "Quad-Ring (X-ring) in a radial gland, Minnesota Rubber"
summary = "A Quad-Ring (X-ring), the four-lobed seal sized like an AS568 O-ring, in a static or reciprocating piston or rod gland: stretch, gland depth, squeeze, gland fill with the Quad-Ring's section area 0.8215 W² and diametral clearance, at nominal size and at the tolerance extremes, checked against Minnesota Rubber's guidance and the extrusion pressure read off Parker's chart."
category = ["Mechanical engineering", "Seals"]
tags = ["Quad-Ring", "X-ring", "four-lobed seal", "gland", "groove", "seal", "squeeze", "gland fill", "stretch", "extrusion", "piston seal", "rod seal", "Minnesota Rubber", "AS568"]
from = "lacewing:mechanical.quadRings"
license = "GPL-3.0-or-later"
inputs = ["side", "ID", "t_ID", "W", "t_W", "D_o", "t_o", "D_i", "t_i", "D_m", "t_m", "G", "t_G", "p", "p_x"]
outputs = ["s", "s_max", "L", "δ", "δ_min", "δ_p", "φ_g", "φ_(g,max)", "E", "E_max"]

[[references]]
work = "Minnesota Rubber, Quad-Ring Brand Design Guide, Sec. 6"
edition = "2003"
locator = "Equations 1 to 8 (stretch, squeeze, gland fill; Quad-Ring section area 0.8215 W²); groove table p. 6-11; size table p. 6-22; stretch and gland fill guidance p. 6-11"

[[references]]
work = "Parker O-Ring Handbook, ORD 5700"
edition = "2018"
locator = "Figure 3-2 (extrusion limits, applied to the Quad-Ring as for an O-ring); Design Charts 4-2 and 5-2-a (diametral clearances)"

[[tests]]
name = "Q4214 in a static rod gland at Minnesota Rubber's depth 0.122 in and width 0.155 in"
source = "Lacewing Lab o_rings_test (Quad-Ring case): ring 0.984 ± 0.010 × 0.139 ± 0.004 in, rod 1.000 in, housing groove 1.244 in, groove 0.155 in (Minnesota Rubber p. 6-11): squeeze (0.139 − 0.122)/0.139 = 12.23 %, fill 0.8215 × 0.139²/(0.122 × 0.155) = 83.94 %, no warnings. The throat, 1.0045 in for Parker's mid-range clearance, and 1000 psi against 1416 psi (Parker Fig. 3-2, 70 Shore A at 0.0045 in, as the Lacewing engine reads it) are added here"
inputs = { side = "2", ID = "0.984 in", t_ID = "0.010 in", W = "0.139 in", t_W = "0.004 in", D_o = "1.244 in", t_o = "0 in", D_i = "1.000 in", t_i = "0 in", D_m = "1.0045 in", t_m = "0 in", G = "0.155 in", t_G = "0 in", p = "1000 psi", p_x = "1416 psi" }
expect = { s = "1.62601626", s_max = "2.66940452", L = "0.122 in", "δ_p" = "12.2302158", "δ_min" = "0.013 in", "φ_g" = "83.9354918", "φ_(g,max)" = "88.8358197", E = "0.0045 in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Q4214 on a 1.020 in rod: stretched and overfilled"
source = "Lacewing Lab engine: the same ring on a 1.020 in rod in a 1.244 in housing groove: stretch 3.66 % (4.72 % at the extremes), over Minnesota Rubber's 3 %, and fill 91.4 % (96.8 %), over its 90 %"
inputs = { side = "2", ID = "0.984 in", t_ID = "0.010 in", W = "0.139 in", t_W = "0.004 in", D_o = "1.244 in", t_o = "0 in", D_i = "1.020 in", t_i = "0 in", D_m = "1.0245 in", t_m = "0 in", G = "0.155 in", t_G = "0 in", p = "1000 psi", p_x = "1416 psi" }
expect = { s = "3.65853659", s_max = "4.72279261", "φ_g" = "91.4297321", "φ_(g,max)" = "96.7675893" }
tolerance = "1e-6"
checks = "fail"
+++

## Quad-Ring in a radial gland

A Quad-Ring (X-ring) is a four-lobed seal made to the AS568 sizes: Q4xxx has the inside diameter and cross-section of O-ring 2-xxx. Its lobes seal with less squeeze than an O-ring, which lowers friction and resists spiral twist in reciprocating service. Minnesota Rubber's Design Guide gives a shallower groove than an O-ring's (a static depth and a shallower dynamic one, p. 6-11) and these rules: above about 3 % stretch choose the next larger size, keep the gland fill to about 90 % at most, and keep some squeeze at every tolerance extreme. Its equations count no loss of cross-section for stretch, so none is taken here. Above 1500 psi it recommends back-up rings.

An extreme takes every dimension at its own worst value, so the bounds are on the safe side where one dimension enters two terms.

### Gland

```calc {description="Gland: 1 piston (male) gland, 2 rod (female) gland"}
side = 2_
```

### Ring

The inside diameter and cross-section with their ± tolerances, from Minnesota Rubber's size table (the AS568 values) or the maker's drawing. The defaults are Q4214's.

```calc {description="Inside diameter of the free ring"}
ID = 0.984 in_
```

```calc {description="± tolerance on the ring's inside diameter"}
t_ID = 0.010 in_
```

```calc {description="Cross-section of the ring, across the lobes"}
W = 0.139 in_
```

```calc {description="± tolerance on the cross-section"}
t_W = 0.004 in_
```

### Gland dimensions

Take a two-sided tolerance as its mid-band and half its width. Minnesota Rubber's groove width is +0.005/−0 in.

```calc {description="Outer diameter: the cylinder bore (piston gland) or the housing groove diameter (rod gland)"}
D_o = 1.244 in_
```

```calc {description="± tolerance on the outer diameter"}
t_o = 0 in_
```

```calc {description="Inner diameter the ring's ID sits on: the groove diameter (piston gland) or the rod (rod gland)"}
D_i = 1.000 in_
```

```calc {description="± tolerance on the inner diameter"}
t_i = 0 in_
```

```calc {description="Mating diameter across the clearance: the piston OD (piston gland) or the throat bore (rod gland)"}
D_m = 1.0045 in_
```

```calc {description="± tolerance on the mating diameter"}
t_m = 0 in_
```

```calc {description="Groove width"}
G = 0.155 in_
```

```calc {description="± tolerance on the groove width"}
t_G = 0 in_
```

### Stretch, squeeze and fill

```calc
> Stretch of the ring's ID, percent
s = 100 (D_i - ID) / ID = ??
> Greatest stretch
s_max = 100 (D_i + t_i - (ID - t_ID)) / (ID - t_ID) = ??
> Gland depth, clearance included
L = (D_o - D_i) / 2 = ?? in
L_min = (D_o - t_o - (D_i + t_i)) / 2 = ?? in
L_max = (D_o + t_o - (D_i - t_i)) / 2 = ?? in
> Squeeze
δ = W - L = ?? in
> Least squeeze
δ_min = W - t_W - L_max = ?? in
> Squeeze, percent
δ_p = 100 δ / W = ??
> Gland fill with the Quad-Ring's section 0.8215 W², Equation 8, percent
φ_g = 100 × 0.8215 W^2 / (L G) = ??
> Greatest fill
φ_(g,max) = 100 × 0.8215 (W + t_W)^2 / (L_min (G - t_G)) = ??
```

### Clearance and extrusion

Minnesota Rubber gives its own chart of permissible clearance against pressure; here the pressure at which the ring starts to extrude through E_max is read off Parker's O-ring chart (Figure 3-2) for the ring's hardness, as for an O-ring.

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
p_x = 1416 psi_
```

### Checks

```calc
@test s_max ≤ 3
```

```calc
@test δ_min > 0 'in'
```

```calc
@test φ_(g,max) ≤ 90
```

```calc
@test p ≤ p_x
```
