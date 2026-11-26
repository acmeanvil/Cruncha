+++
cruncha-template = 1
id = "structural.column.alignmentK"
version = "1.0.0"
title = "Effective length factor K from the alignment charts, AISC 360"
summary = "Effective length factor of a frame column from the joint stiffness ratios at its ends, by the alignment-chart equations for a braced frame (sidesway inhibited) and a moment frame (sidesway uninhibited)."
category = ["Structural engineering", "Members"]
tags = ["effective length", "K factor", "alignment chart", "nomograph", "G factor", "braced frame", "sway frame", "moment frame", "column", "AISC 360", "Appendix 7", "stability"]
from = "lacewing:structural.columnSlenderness"
license = "GPL-3.0-or-later"
inputs = ["G_A", "G_B"]
outputs = ["K_b", "K_s"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings, Commentary"
edition = "2022"
locator = "Appendix 7, Sec. 7.2: Eqs. C-A-7-1 (sidesway inhibited), C-A-7-2 (sidesway uninhibited), C-A-7-3 (G); Figs. C-A-7.1 and C-A-7.2"

[[tests]]
name = "G_A = G_B = 1: braced 0.77, sway 1.32"
source = "Alignment charts, AISC 360-22 Commentary Figs. C-A-7.1 and C-A-7.2, read as 0.77 and 1.32 (Lacewing Lab columns_test); the equations solved to 1e-9 in Python give 0.774265 and 1.317275"
inputs = { G_A = "1", G_B = "1" }
expect = { K_b = "0.774265", K_s = "1.317275" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Pinned base (G 10) under a stiff joint (G 1)"
source = "By the equations, solved by bisection in Python: braced 0.859922, sway 1.902969 (the charts read about 0.86 and 1.9)"
inputs = { G_A = "1", G_B = "10" }
expect = { K_b = "0.859922", K_s = "1.902969" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Near-fixed ends (G 0.1): braced 0.55, sway 1.03"
source = "By the equations, solved by bisection in Python: braced 0.548698, sway 1.033305; they tend to the ideal 0.5 and 1.0 as G goes to 0 (Lacewing Lab columns_test)"
inputs = { G_A = "0.1", G_B = "0.1" }
expect = { K_b = "0.548698", K_s = "1.033305" }
tolerance = "1e-5"
checks = "pass"
+++

## Effective length factor from the alignment charts

The alignment charts (nomographs) of the AISC 360 Commentary give the effective length factor K of a column in a rectangular frame from the stiffness of the members at each of its ends. G at a joint is Σ(EI/L) of the columns meeting there over Σ(EI/L) of the girders (Eq. C-A-7-3). The template solves the charts' equations for K directly, for both cases:

- **braced frame** (sidesway inhibited, Eq. C-A-7-1): K between 0.5 and 1;
- **moment frame** (sidesway uninhibited, Eq. C-A-7-2): K above 1.

The charts assume an idealised frame: elastic behaviour, prismatic members, all columns buckling together, girders restrained equally at both ends. Adjust G for girder end conditions or inelastic columns as the Commentary describes. For a column base the Commentary suggests G = 10 for a pinned base and G = 1 for a fixed one, rather than the theoretical infinity and zero; G must be greater than zero here.

### Joint stiffness ratios

```calc {description="Joint stiffness ratio Σ(EI/L)columns / Σ(EI/L)girders at the top"}
G_A = 1_
```

```calc {description="Joint stiffness ratio at the bottom (pinned base 10, fixed base 1)"}
G_B = 1_
```

### Braced frame

```calc
function alignBraced(K, G_A, G_B)
    x = π / K
    return G_A G_B / 4 x^2 + (G_A + G_B) / 2 (1 - x / tan(x)) + 2 tan(x / 2) / x - 1
end
```

```calc
function kBraced(G_A, G_B)
    lo = 0.500000001
    hi = 0.999999999999
    f_lo = alignBraced(lo, G_A, G_B)
    for i in 1:60
        m = (lo + hi) / 2
        f_m = alignBraced(m, G_A, G_B)
        if f_m f_lo > 0
            lo = m
            f_lo = f_m
        else
            hi = m
        end
    end
    return (lo + hi) / 2
end
```

```calc {description="Effective length factor, sidesway inhibited, Eq. C-A-7-1"}
K_b = kBraced(G_A, G_B) = ??
```

### Moment frame

```calc
function alignSway(K, G_A, G_B)
    x = π / K
    return (G_A G_B x^2 - 36) / (6 (G_A + G_B)) - x / tan(x)
end
```

```calc
function kSway(G_A, G_B)
    lo = 1.000000001
    hi = 100
    f_lo = alignSway(lo, G_A, G_B)
    for i in 1:60
        m = (lo + hi) / 2
        f_m = alignSway(m, G_A, G_B)
        if f_m f_lo > 0
            lo = m
            f_lo = f_m
        else
            hi = m
        end
    end
    return (lo + hi) / 2
end
```

```calc {description="Effective length factor, sidesway uninhibited, Eq. C-A-7-2"}
K_s = kSway(G_A, G_B) = ??
```

### Check

```calc
@test G_A > 0 and G_B > 0
```
