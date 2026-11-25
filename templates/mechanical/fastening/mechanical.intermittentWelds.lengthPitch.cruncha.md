+++
cruncha-template = 1
id = "mechanical.intermittentWelds.lengthPitch"
version = "1.0.0"
title = "Intermittent fillet welds: length and pitch for a continuous weld's strength"
summary = "The share R of a continuous fillet weld that a larger leg laid intermittently must cover, the segment length it needs at a chosen pitch, and a check that the chosen length and pitch carry what the continuous weld would."
category = ["Mechanical engineering", "Fastening"]
tags = ["weld", "fillet weld", "intermittent weld", "stitch weld", "skip weld", "pitch", "leg size", "Schaum"]
from = "lacewing:mechanical.intermittentWelds"
license = "GPL-3.0-or-later"
inputs = ["w_c", "w_i", "l_i", "p"]
outputs = ["R", "l_req", "R_used", "q_ratio"]

[[references]]
work = "Schaum's Outline of Machine Design (Hall, Holowenko, Laughlin)"
edition = "1st"
locator = "Ch. 25, Welding: intermittent welds, pp. 305-306 (table of lengths and centre distances by percentage of a continuous weld); solved problems 10 and 17"

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J2.2b (minimum length of an intermittent fillet weld segment)"

[[tests]]
name = "Schaum problem 17: 0.180 in continuous, 3/8 in used, 3 in on 6 in centres"
source = "Lacewing Lab weld_test, after Schaum's Machine Design ch. 25 problem 17: R = 0.180/0.375 = 48 %, which the 50 % row covers, with 3 in welds on 6 in centres among its choices"
inputs = { w_c = "0.180 in", w_i = "0.375 in", l_i = "3 in", p = "6 in" }
expect = { R = "0.48", l_req = "2.88 in", R_used = "0.5" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Schaum problem 10: 0.164 in continuous, 0.333 in used, 4 in on 8 in centres"
source = "Lacewing Lab weld_test, after Schaum's Machine Design ch. 25 problem 10: R = 0.164/0.333 = 49.2 %, on the 50 % row"
inputs = { w_c = "0.164 in", w_i = "0.333 in", l_i = "4 in", p = "8 in" }
expect = { R = "0.492492", R_used = "0.5" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Problem 17's welds at 2 in on 5 in centres: too little weld"
source = "By hand: 2/5 = 40 % covers less than R = 48 %; the intermittent weld carries 0.40/0.48 = 83.3 % of the continuous one's load"
inputs = { w_c = "0.180 in", w_i = "0.375 in", l_i = "2 in", p = "5 in" }
expect = { R_used = "0.4", q_ratio = "0.833333333" }
tolerance = "1e-8"
checks = "fail"

[[tests]]
name = "A continuous weld that needs more than 3/4 of the larger leg: weld continuously"
source = "By hand: R = 0.3125/0.375 = 83.3 %, above the 75 % where Schaum's table ends; 4 in on 4.5 in centres covers it, but the check on R fails"
inputs = { w_c = "0.3125 in", w_i = "0.375 in", l_i = "4 in", p = "4.5 in" }
expect = { R = "0.833333333" }
tolerance = "1e-8"
checks = "fail"
+++

## Intermittent fillet welds

When the fillet a continuous weld needs is smaller than is practical to lay, or than the minimum size for the plates, a larger fillet in short lengths can carry the same load. A fillet's strength per length is in proportion to its leg, so a leg w_i covering a share R = w_c/w_i of the length carries what a continuous leg w_c does (Schaum pp. 305-306). Schaum tabulates standard segment lengths and centre distances (pitches) from 16 % to 75 % of a continuous weld; above 75 %, weld continuously.

Each segment is assumed to carry its full leg over its whole length. Keep the segments at least four times the leg and 1.5 in long (AISC 360 J2.2b), and the clear gap between them within what the design code allows for the parts' stability.

### Welds

```calc {description="Fillet leg a continuous weld along the whole length would need"}
w_c = 0.180 in_
```

```calc {description="Larger fillet leg to be laid intermittently"}
w_i = 0.375 in_
```

```calc {description="Length of each weld segment"}
l_i = 3 in_
```

```calc {description="Pitch: centre-to-centre distance of the segments"}
p = 6 in_
```

### Share of a continuous weld

```calc
> Share of the length the intermittent weld must cover, w_c / w_i
R = w_c / w_i = ??
> Segment length needed at the chosen pitch
l_req = R p = ?? in
> Share the chosen length and pitch cover
R_used = l_i / p = ??
> Load the intermittent weld carries over the continuous weld's
q_ratio = R_used / R = ??
```

### Checks

```calc
@test R_used ≥ R
```

```calc
@test R ≤ 0.75
```

```calc
@test l_i ≥ max(4 w_i, 1.5 'in')
```
