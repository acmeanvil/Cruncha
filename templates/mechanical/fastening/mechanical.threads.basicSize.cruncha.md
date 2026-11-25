+++
cruncha-template = 1
id = "mechanical.threads.basicSize"
version = "1.0.0"
title = "Thread size: basic diameters and tensile-stress area (UN, M)"
summary = "Basic pitch and minor diameters, tensile-stress and root areas of a 60° Unified inch or ISO metric thread from its major diameter and pitch, and the axial load it carries at a given strength."
category = ["Mechanical engineering", "Fastening"]
tags = ["thread", "screw thread", "tensile stress area", "UNC", "UNF", "Unified", "ISO metric", "pitch diameter", "minor diameter", "bolt", "proof load"]
from = "lacewing:mechanical.threads"
license = "GPL-3.0-or-later"
inputs = ["d", "P", "UN", "S"]
outputs = ["d_p", "d_r", "D_1", "A_t", "A_r", "F_S"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-1, Fig. 8-2 (basic profile); the notes to Tables 8-1 and 8-2 (minor diameter and tensile-stress area)"

[[references]]
work = "ASME B1.1 Unified Inch Screw Threads (UN and UNR Thread Form)"
edition = "2019"
locator = "Tensile stress area formula; dimension tables for the standard sizes"

[[references]]
work = "ASME B1.13M Metric Screw Threads: M Profile"
edition = "2005 (R2020)"
locator = "Appendix B-1 (tensile stress area); dimension tables for the standard sizes"

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Bolts and Nuts, Eq. 2a (tensile stress area of Unified threads)"

[[tests]]
name = "M12 × 1.75 coarse"
source = "Lacewing Lab threads_test, after Shigley 10th Table 8-1: At = 84.3 mm²; pitch diameter 10.863 mm"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", S = "640 MPa" }
expect = { A_t = "84.3 mm²", d_p = "10.863 mm" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "1/2-13 UNC"
source = "Lacewing Lab threads_test, after Shigley 10th Table 8-2: At = 0.1419 in²"
inputs = { d = "0.5 in", P = "0.0769230769 in", UN = "1", S = "92 ksi" }
expect = { A_t = "0.1419 in²" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "1/4-28 UNF"
source = "Shigley 10th Table 8-2: At = 0.0364 in², Ar = 0.0326 in²"
inputs = { d = "0.25 in", P = "0.0357142857 in", UN = "1", S = "92 ksi" }
expect = { A_t = "0.0364 in²", A_r = "0.0326 in²" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "M12 × 1.75 at 640 MPa"
source = "Lacewing Lab threads_test: 84.3 mm² × 640 MPa = 53 952 N"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", S = "640 MPa" }
expect = { F_S = "53.95 kN" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: M12 × 1.75 exactly"
source = "By hand: dp = 12 − 0.649519·1.75 = 10.863342 mm; dr = 12 − 1.226869·1.75 = 9.852979 mm; At = π/4·((dp + dr)/2)² = π/4·10.358161² = 84.26654 mm²; D1 = 12 − 1.082532·1.75 = 10.105569 mm"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", S = "640 MPa" }
expect = { d_p = "10.86334175 mm", d_r = "9.85297925 mm", A_t = "84.266538 mm²", D_1 = "10.105569 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## Thread size: basic diameters and tensile-stress area

The basic dimensions of a 60° screw thread, Unified inch (UN) or ISO metric (M), from its major diameter and pitch, and the axial load the external thread carries at a stress over its tensile-stress area. The tensile-stress area is the area of a plain rod as strong in tension as the threaded one; it lies between the pitch and minor diameters.

These are the **basic** sizes, without tolerances. The limits of size of a class (2A/2B, 6g/6H and so on) are in the thread's standard: ASME B1.1 for Unified threads, ASME B1.13M or ISO 965-1 for metric ones. For stripping strength and the stresses under load, use the thread strength template with the limits of size from those tables.

### Size

```calc {description="Basic major (nominal) diameter"}
d = 12 mm_
```

For an inch thread of n threads per inch, the pitch is 1/n: 0.0769231 in for 13 threads per inch.

```calc {description="Pitch, the axial distance between adjacent threads"}
P = 1.75 mm_
```

```calc {description="Thread form: 1 for Unified inch (UN), 0 for ISO metric (M)"}
UN = 0_
```

```calc {description="Strength the load is taken at: proof, yield or tensile"}
S = 640 MPa_
```

### Basic diameters

```calc {description="Height of the fundamental triangle, Fig. 8-2"}
H = sqrt(3) / 2 P = ?? mm
```

```calc {description="Basic pitch diameter"}
d_p = d - 0.649519 P = ?? mm
```

The minor diameter used for the stress area is the external thread's: d − 1.299038P for Unified threads and d − 1.226869P for metric ones (the notes to Shigley's Tables 8-1 and 8-2).

```calc {description="Minor (root) diameter of the external thread"}
d_r = { d - 1.299038 P if UN == 1; d - 1.226869 P otherwise } = ?? mm
```

```calc {description="Basic minor diameter of the internal thread, 5H/4 below the major"}
D_1 = d - 1.082532 P = ?? mm
```

### Areas

The tensile-stress area is that of the mean of the pitch and minor diameters. For a Unified thread this is ASME B1.1's 0.7854(d − 0.9743/n)²; for a metric thread B1.13M's 0.7854(d − 0.9382P)².

```calc {description="Tensile-stress area"}
A_t = π / 4 ((d_p + d_r) / 2)^2 = ?? mm²
```

```calc {description="Area at the minor diameter"}
A_r = π / 4 d_r^2 = ?? mm²
```

### Load

```calc {description="Axial load at the strength S over the tensile-stress area"}
F_S = A_t S = ?? kN
```
