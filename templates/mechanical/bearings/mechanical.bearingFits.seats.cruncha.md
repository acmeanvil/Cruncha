+++
cruncha-template = 1
id = "mechanical.bearingFits.seats"
version = "1.0.0"
title = "Rolling bearing shaft and housing fits"
summary = "The fits of a radial rolling bearing on its shaft and in its housing from the ring tolerances and the chosen seat tolerance classes: the limits of the bearing's mean bore and outside diameter, of the shaft seat and the housing bore, the largest and smallest interference of the inner ring and clearance of the outer ring, and the load as a fraction of the dynamic rating that the class recommendations are given for."
category = ["Mechanical engineering", "Bearings"]
tags = ["bearing fit", "rolling bearing", "shaft fit", "housing fit", "interference", "clearance", "tolerance", "ABMA 7", "ABMA 20", "ABEC", "ISO 286", "Machinery's Handbook"]
from = "lacewing:mechanical.bearingFits"
license = "GPL-3.0-or-later"
inputs = ["d", "D", "Δ_dmp", "Δ_Dmp", "es", "ei", "ES", "EI", "P", "C"]
outputs = ["d_(b,max)", "d_(b,min)", "d_(s,max)", "d_(s,min)", "i_max", "i_min", "D_(o,max)", "D_(o,min)", "D_(h,max)", "D_(h,min)", "c_max", "c_min", "r_P"]

[[references]]
work = "Machinery's Handbook"
edition = "26th"
locator = "Ball and Roller Bearings: Tables 1 to 5 (ANSI/ABMA 20 ring tolerances, Δdmp and ΔDmp), the load classes and Tables 13 and 14 (ANSI/ABMA 7 shaft and housing tolerance classes), Table 16 (seat limits); Tables 22 to 26 (ISO tolerances and fundamental deviations)"

[[references]]
work = "ISO 286-2 Geometrical product specifications — ISO code system for tolerances on linear sizes — Part 2: Tables of standard tolerance classes and limit deviations"
edition = "ISO 286-2:2010"
locator = "Limit deviations es, ei of shafts and ES, EI of holes for each tolerance class and size range"

[[tests]]
name = "6208 ball bearing, ABEC-1, normal load: k5 shaft and H7 housing"
source = "Lacewing Lab inch_bearing_fits_test, after Machinery's Handbook 26th Tables 1, 13, 14 and the ISO deviations: 40 mm bore 0/−12 µm on k5 (+13/+2 µm), interference 2 to 25 µm; 80 mm outside diameter 0/−13 µm in H7 (+30/0 µm), clearance 0 to 43 µm. Load 3 kN on C10 = 30.7 kN (Shigley Table 11-2, 02-40), 0.098 C: a normal load"
inputs = { d = "40 mm", D = "80 mm", "Δ_dmp" = "-12 µm", "Δ_Dmp" = "-13 µm", es = "13 µm", ei = "2 µm", ES = "30 µm", EI = "0 µm", P = "3 kN", C = "30.7 kN" }
expect = { i_max = "25 µm", i_min = "2 µm", c_max = "43 µm", c_min = "0 µm", r_P = "0.0977198697" }
tolerance = "1e-9"

[[tests]]
name = "Hand check: the same bearing in a J6 housing (transition fit)"
source = "By hand: 80 mm outside diameter 0/−13 µm in a J6 bore of +13/−6 µm: the housing bore runs 79.994 to 80.013 mm, the ring 79.987 to 80.000 mm, clearance from −6 µm (6 µm interference) to 26 µm"
inputs = { d = "40 mm", D = "80 mm", "Δ_dmp" = "-12 µm", "Δ_Dmp" = "-13 µm", es = "13 µm", ei = "2 µm", ES = "13 µm", EI = "-6 µm" }
expect = { "D_(h,max)" = "80.013 mm", "D_(h,min)" = "79.994 mm", "D_(o,min)" = "79.987 mm", c_max = "26 µm", c_min = "-6 µm" }
tolerance = "1e-9"
+++

## Rolling bearing shaft and housing fits

A metric radial ball or roller bearing's rings are made to tolerance on their mean diameters: the mean bore and outside diameter are never above nominal and up to Δ_dmp and Δ_Dmp below it (ANSI/ABMA 20, Machinery's Handbook Tables 1 to 5, by the ring tolerance class ABEC-1/RBEC-1 to ABEC-9 and the size). The seats are ordinary ISO 286 tolerance classes, chosen by the way the rings turn and the load. This template works out the fits from those deviations, which are read from the tables and entered here; it does not choose the classes, compute the press-fit pressure, or the bearing's internal clearance after mounting.

### Choosing the classes

ANSI/ABMA 7 (Machinery's Tables 13 and 14) recommends classes for ABEC-1 and RBEC-1 bearings on solid steel shafts in cast iron or steel housings; hollow or nonferrous shafts and nonferrous housings may need tighter fits.

- **Inner ring rotating relative to the load**, or a load of indeterminate direction: an interference fit, so the ring cannot creep on the shaft; tighter for heavier loads, larger shafts and roller bearings (j5, k5, m5 and up for ball bearings).
- **Inner ring stationary relative to the load**: a loose fit, g6 where the ring must slide along the shaft, h6 otherwise. Pure thrust: j6.
- **Outer ring stationary relative to the load** and free to slide axially (the floating bearing): H6 or H7 in a solid housing, H7 in a split one, G7 where heat flows in through the shaft. Shock with complete unloading, or a load of indeterminate direction: J6, K6 for heavy shock.
- **Outer ring rotating relative to the load** (a wheel hub on a fixed axle): M6, N6 or P6 by the load, an interference fit.

The load classes are fractions of the bearing's basic dynamic load rating C: for ball bearings light up to 0.075C, normal 0.075C to 0.15C and heavy above 0.15C; roller bearings have their own limits in the same tables. Too much interference takes up the bearing's internal clearance.

```calc {description="Equivalent load on the bearing"}
P = 3 kN_
```

```calc {description="Basic dynamic load rating of the bearing"}
C = 30.7 kN_
```

```calc {description="Load as a fraction of the dynamic rating, for the load class"}
r_P = P / C = ??
```

### Inner ring on the shaft

The bearing bore is the hole of this fit.

```calc {description="Nominal bore of the bearing, the basic size of the shaft seat"}
d = 40 mm_
```

```calc {description="Lower deviation of the mean bore, ABMA 20 (Machinery's Tables 1 to 5)"}
Δ_dmp = -12 µm_
```

```calc {description="Upper deviation es of the shaft seat's tolerance class, ISO 286"}
es = 13 µm_
```

```calc {description="Lower deviation ei of the shaft seat's tolerance class, ISO 286"}
ei = 2 µm_
```

```calc
> Largest mean bore
d_(b,max) = d = ?? mm
> Smallest mean bore
d_(b,min) = d + Δ_dmp = ?? mm
> Largest shaft seat
d_(s,max) = d + es = ?? mm
> Smallest shaft seat
d_(s,min) = d + ei = ?? mm
> Largest interference of the inner ring
i_max = d_(s,max) - d_(b,min) = ?? µm
> Smallest interference of the inner ring (negative: a clearance)
i_min = d_(s,min) - d_(b,max) = ?? µm
```

### Outer ring in the housing

The bearing's outside diameter is the shaft of this fit.

```calc {description="Nominal outside diameter of the bearing, the basic size of the housing bore"}
D = 80 mm_
```

```calc {description="Lower deviation of the mean outside diameter, ABMA 20 (Machinery's Tables 1 to 5)"}
Δ_Dmp = -13 µm_
```

```calc {description="Upper deviation ES of the housing bore's tolerance class, ISO 286"}
ES = 30 µm_
```

```calc {description="Lower deviation EI of the housing bore's tolerance class, ISO 286"}
EI = 0 µm_
```

```calc
> Largest mean outside diameter
D_(o,max) = D = ?? mm
> Smallest mean outside diameter
D_(o,min) = D + Δ_Dmp = ?? mm
> Largest housing bore
D_(h,max) = D + ES = ?? mm
> Smallest housing bore
D_(h,min) = D + EI = ?? mm
> Largest clearance of the outer ring
c_max = D_(h,max) - D_(o,min) = ?? µm
> Smallest clearance of the outer ring (negative: an interference)
c_min = D_(h,min) - D_(o,max) = ?? µm
```
