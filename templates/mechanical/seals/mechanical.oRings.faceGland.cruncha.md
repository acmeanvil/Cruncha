+++
cruncha-template = 1
id = "mechanical.oRings.faceGland"
version = "1.0.0"
title = "O-ring in a face (axial) gland, Parker"
summary = "An O-ring sealing axially in a face groove: squeeze and gland fill at nominal size and at the tolerance extremes of the ring's cross-section, the groove depth and the groove width, checked against the Parker O-Ring Handbook's static limits."
category = ["Mechanical engineering", "Seals"]
tags = ["O-ring", "face seal", "axial seal", "flange seal", "gland", "groove", "squeeze", "gland fill", "static seal", "AS568", "Parker", "elastomer"]
from = "lacewing:mechanical.oRings"
license = "GPL-3.0-or-later"
inputs = ["W", "t_W", "L", "t_L", "G", "t_G"]
outputs = ["δ", "δ_min", "δ_p", "δ_(p,max)", "φ_g", "φ_(g,min)", "φ_(g,max)"]

[[references]]
work = "Parker O-Ring Handbook, ORD 5700"
edition = "2018"
locator = "Secs. 3.6 and 3.7 (squeeze and gland fill: the limits); Design Chart 4-3 (face seal glands); Table 9-1 (AS568 sizes)"

[[tests]]
name = "2-214 in a groove 0.136 in deep: too little squeeze"
source = "Lacewing Lab o_rings_test: ring 0.139 ± 0.004 in (Parker Table 9-1), groove 0.136 in deep and 0.19 in wide; squeeze 0.003 in nominal and −0.001 in for the thinnest ring, under Parker's 0.007 in, and fill 55.4 % at the least, under 60 %. Values are the Lacewing engine's"
inputs = { W = "0.139 in", t_W = "0.004 in", L = "0.136 in", t_L = "0 in", G = "0.19 in", t_G = "0 in" }
expect = { "δ" = "0.003 in", "δ_min" = "-0.001 in", "δ_p" = "2.15827338", "φ_g" = "58.7255337", "φ_(g,min)" = "55.3942784" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "2-214 in a groove 0.107 ± 0.002 in deep, 0.19 ± 0.005 in wide"
source = "By hand, and the Lacewing engine: squeeze 0.032 in (23.02 %), least 0.026 in, greatest 26.57 %; fill 74.64 %, from 67.34 to 82.68 %: inside Parker's limits (Chart 4-3's depth for this section is 0.101 to 0.107 in)"
inputs = { W = "0.139 in", t_W = "0.004 in", L = "0.107 in", t_L = "0.002 in", G = "0.19 in", t_G = "0.005 in" }
expect = { "δ" = "0.032 in", "δ_min" = "0.026 in", "δ_p" = "23.0215827", "δ_(p,max)" = "26.5734266", "φ_g" = "74.6417999", "φ_(g,min)" = "67.3435969", "φ_(g,max)" = "82.6800877" }
tolerance = "1e-6"
checks = "pass"
+++

## O-ring in a face gland

An O-ring in a groove cut in a flat face, squeezed axially by the mating face: a flange, a cover or a plug. The ring is not stretched or compressed in diameter by much (Parker puts the groove's outer diameter at the ring's for internal pressure, its inner diameter at the ring's for external pressure), so its cross-section is taken as made. The checks are Parker's for a static seal (Secs. 3.6 and 3.7): a squeeze of at least 0.007 in and at most 30 %, and a gland fill of 60 to 85 %. With the faces metal to metal there is no extrusion gap to check. For radial glands use the radial gland template.

### Ring

The cross-section and its ± tolerance, from SAE AS568 (Parker Table 9-1) or the maker's drawing. The default is the 2-1xx and 2-2xx series' 0.139 in.

```calc {description="Cross-section diameter of the ring"}
W = 0.139 in_
```

```calc {description="± tolerance on the cross-section"}
t_W = 0.004 in_
```

### Groove

Parker's Design Chart 4-3 gives the recommended depth and width for each cross-section, with wider grooves for gases than for liquids. Take a two-sided tolerance as its mid-band and half its width.

```calc {description="Groove depth (the gland depth of a face seal)"}
L = 0.107 in_
```

```calc {description="± tolerance on the groove depth"}
t_L = 0.002 in_
```

```calc {description="Groove width (radial)"}
G = 0.19 in_
```

```calc {description="± tolerance on the groove width"}
t_G = 0.005 in_
```

### Squeeze and fill

```calc
> Squeeze
δ = W - L = ?? in
> Least squeeze: thinnest ring, deepest groove
δ_min = W - t_W - (L + t_L) = ?? in
> Squeeze as a percentage of the cross-section
δ_p = 100 δ / W = ??
> Greatest squeeze, percent: thickest ring, shallowest groove
δ_(p,max) = 100 (W + t_W - (L - t_L)) / (W + t_W) = ??
> Gland fill: the ring's section over the groove's, percent
φ_g = 100 π W^2 / (4 L G) = ??
φ_(g,min) = 100 π (W - t_W)^2 / (4 (L + t_L) (G + t_G)) = ??
φ_(g,max) = 100 π (W + t_W)^2 / (4 (L - t_L) (G - t_G)) = ??
```

### Checks

```calc
@test δ_min ≥ 0.007 'in'
```

```calc
@test δ_(p,max) ≤ 30
```

```calc
@test 60 ≤ φ_(g,min)
```

```calc
@test φ_(g,max) ≤ 85
```
