+++
cruncha-template = 1
id = "mechanical.lipSeals.speed"
version = "1.0.0"
title = "Rotary lip seal: surface speed and pressure (SKF)"
summary = "A spring-loaded radial shaft (lip) seal: the shaft's surface speed at the lip against the permissible speed for the lip material read off SKF's diagram (after DIN 3760), and the pressure across the seal against its rating at 5 m/s."
category = ["Mechanical engineering", "Seals"]
tags = ["lip seal", "radial shaft seal", "rotary seal", "oil seal", "surface speed", "permissible speed", "nitrile", "NBR", "FKM", "polyacrylate", "DIN 3760", "SKF"]
from = "lacewing:mechanical.lipSeals"
license = "GPL-3.0-or-later"
inputs = ["d", "N", "v_p", "p", "p_r"]
outputs = ["v"]

[[references]]
work = "SKF Industrial shaft seals"
edition = "2021"
locator = "Diagram 5 (permissible circumferential speeds of NBR, ACM and FKM lips, after DIN 3760); p. 26 (pressure ratings of standard and CRW5/CRWA5 seals); p. 79 (seals under pressure); pp. 80-84 (counterface and housing)"

[[references]]
work = "DIN 3760, Radial-Wellendichtringe (rotary shaft lip seals)"
edition = "1996"
locator = "permissible speeds by lip material"

[[tests]]
name = "Nitrile lip on a 50 mm shaft at 2500 rpm, no pressure"
source = "Lacewing Lab o_rings_test (lip seals): v = π × 0.05 × 2500/60 = 6.545 m/s, under the 6.88 m/s SKF Diagram 5 permits a nitrile lip at 50 mm (as the Lacewing engine reads the diagram)"
inputs = { d = "50 mm", N = "2500 rpm", v_p = "6.88 m/s", p = "0 MPa", p_r = "0.07 MPa" }
expect = { v = "6.54498469 m/s" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same seal at 6000 rpm: too fast for nitrile"
source = "Lacewing Lab o_rings_test (lip seals): v = 15.708 m/s exceeds the 6.88 m/s of SKF Diagram 5 for nitrile at 50 mm"
inputs = { d = "50 mm", N = "6000 rpm", v_p = "6.88 m/s", p = "0 MPa", p_r = "0.07 MPa" }
expect = { v = "15.7079633 m/s" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Fluoro rubber lip at 1000 rpm with 0.1 MPa across a standard seal"
source = "Lacewing Lab o_rings_test (lip seals): 0.1 MPa is over the 0.07 MPa SKF rates standard seals for at 5 m/s (p. 26); v = 2.618 m/s against 25.2 m/s on Diagram 5 for FKM at 50 mm"
inputs = { d = "50 mm", N = "1000 rpm", v_p = "25.2 m/s", p = "0.1 MPa", p_r = "0.07 MPa" }
expect = { v = "2.61799388 m/s" }
tolerance = "1e-6"
checks = "fail"
+++

## Rotary lip seal: surface speed and pressure

A spring-loaded radial lip seal on a rotating shaft. Its speed limit is set by the heat at the lip, so it depends on the lip material and the shaft diameter: SKF's Diagram 5 (after DIN 3760) gives the permissible circumferential speed of nitrile (NBR), polyacrylate (ACM) and fluoro rubber (FKM) lips up to a 200 mm shaft, with no pressure across the seal. With pressure the lip load, friction and heat rise and the diagram no longer applies: SKF rates standard seals for 0.07 MPa and its CRW5 and CRWA5 pressure profile seals for 0.34 MPa, both at up to 5 m/s.

This does not choose the seal. SKF's counterface requirements also apply: shaft to ISO h11, Ra 0.2 to 0.5 µm, at least 30 HRC (45 HRC where it could be damaged in handling), housing bore to H8.

```calc {description="Shaft diameter at the lip"}
d = 50 mm_
```

```calc {description="Shaft speed"}
N = 2500 rpm_
```

```calc {description="Permissible surface speed for the lip material at this diameter, SKF Diagram 5"}
v_p = 6.88 m/s_
```

```calc {description="Pressure across the seal (gauge), 0 with none"}
p = 0 MPa_
```

```calc {description="Pressure rating of the seal at 5 m/s, SKF p. 26: 0.07 MPa standard, 0.34 MPa CRW5/CRWA5"}
p_r = 0.07 MPa_
```

```calc {description="Surface speed of the shaft at the lip"}
v = π d N / (1 'rev') = ?? m/s
```

### Checks

```calc
@test v ≤ v_p
```

```calc
@test p ≤ p_r
```

```calc
@test p == 0 'MPa' or v ≤ 5 'm/s'
```
