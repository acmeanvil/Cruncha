+++
cruncha-template = 1
id = "mechanics.impact.central"
version = "1.0.0"
title = "Central impact of two bodies"
summary = "Two bodies colliding along the line through their centres of gravity, with a coefficient of restitution: the velocities after impact (momentum kept, relative speed times e), the kinetic energy lost and its share, and the impulse each body takes."
category = ["Mechanical engineering", "Mechanics"]
tags = ["impact", "collision", "coefficient of restitution", "momentum", "impulse", "elastic", "plastic", "kinetic energy", "energy loss", "Den Hartog"]
from = "lacewing:mechanics.impact"
license = "GPL-3.0-or-later"
inputs = ["m_1", "m_2", "V_1", "V_2", "e"]
outputs = ["v_1", "v_2", "v_c", "ΔT", "η_T", "J"]

[[references]]
work = "Den Hartog, J.P., Mechanics (Dover reprint)"
edition = "1948"
locator = "§53, central impact: Eqs. 28 (momentum), 28a (restitution), 29 (velocities after) and 30 (energy lost)"

[[tests]]
name = "Elastic impact of equal masses: they swap velocities"
source = "Lacewing Lab mechanics_test, after Den Hartog §53 (e = 1): 2 kg at 3 m/s meets 2 kg at −1 m/s, after −1 and 3 m/s, no energy lost"
inputs = { m_1 = "2 kg", m_2 = "2 kg", V_1 = "3 m/s", V_2 = "-1 m/s", e = "1" }
expect = { v_1 = "-1 m/s", v_2 = "3 m/s", "ΔT" = "0 J" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Plastic impact: the bodies move on together"
source = "Lacewing Lab mechanics_test, after Den Hartog Eqs. 29-30 (e = 0): 3 kg at 4 m/s strikes 1 kg at rest, both 3 m/s after, ΔT = ½·3·16 − ½·4·9 = 6 J"
inputs = { m_1 = "3 kg", m_2 = "1 kg", V_1 = "4 m/s", V_2 = "0 m/s", e = "0" }
expect = { v_1 = "3 m/s", v_2 = "3 m/s", v_c = "3 m/s", "ΔT" = "6 J", "η_T" = "0.25", J = "3 N·s" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: Lacewing Lab's opening case, 2 kg at 4 m/s on 5 kg at rest, e = 0.6"
source = "By hand, Eqs. 29-30: v1 = (8 − 12)/7 = −0.5714286 m/s, v2 = 2 × 4 × 1.6/7 = 1.8285714 m/s, ΔT = 0.32 × (10/7) × 16 = 7.3142857 J of 16 J (45.71 %), impulse 5 × 1.8285714 = 9.1428571 N·s"
inputs = { m_1 = "2 kg", m_2 = "5 kg", V_1 = "4 m/s", V_2 = "0 m/s", e = "0.6" }
expect = { v_1 = "-0.57142857 m/s", v_2 = "1.8285714 m/s", "ΔT" = "7.3142857 J", "η_T" = "0.45714286", J = "9.1428571 N·s" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Body 1 can't catch body 2"
source = "By hand: V1 = 1 m/s behind V2 = 2 m/s: they never meet"
inputs = { m_1 = "2 kg", m_2 = "5 kg", V_1 = "1 m/s", V_2 = "2 m/s", e = "0.6" }
expect = { v_c = "1.7142857 m/s" }
tolerance = "1e-7"
checks = "fail"
+++

## Central impact of two bodies

Two bodies moving along one line collide with the contact force along the line through both centres of gravity, so neither starts to spin (Den Hartog §53). Momentum is kept (Eq. 28), and the relative speed of separation is e times that of approach (Eq. 28a). Together they give the velocities after (Eq. 29) and the kinetic energy lost (Eq. 30), which goes into heat, vibration and permanent deformation.

e = 1 is an elastic impact (equal masses swap velocities); e = 0 a plastic one (the bodies move on together). The coefficient depends on the materials, the speed and the shape; it can be measured by dropping one on the other, e = √(rebound height / drop height).

Velocities are positive to the right, with body 1 on the left: it must be moving faster than body 2 to catch it. A body moving left has a negative velocity.

```calc {description="Mass of body 1 (on the left)"}
m_1 = 2 kg_
```

```calc {description="Mass of body 2 (on the right)"}
m_2 = 5 kg_
```

```calc {description="Velocity of body 1 before impact, + to the right"}
V_1 = 4 m/s_
```

```calc {description="Velocity of body 2 before impact, + to the right (0 at rest)"}
V_2 = 0 m/s_
```

```calc {description="Coefficient of restitution, 0 (plastic) to 1 (elastic)"}
e = 0.6_
```

### After impact

```calc
> Velocity of body 1 after impact, Eq. 29
v_1 = (m_1 V_1 - m_2 e V_1 + m_2 V_2 (1 + e)) / (m_1 + m_2) = ?? m/s
> Velocity of body 2 after impact, Eq. 29
v_2 = (m_2 V_2 - m_1 e V_2 + m_1 V_1 (1 + e)) / (m_1 + m_2) = ?? m/s
> Velocity of the common centre of gravity, unchanged by the impact
v_c = (m_1 V_1 + m_2 V_2) / (m_1 + m_2) = ?? m/s
```

### Energy and impulse

```calc
> Kinetic energy before impact
T_b = m_1 V_1^2 / 2 + m_2 V_2^2 / 2 = ?? J
> Kinetic energy lost, Eq. 30
ΔT = (1 - e × e) / 2 × m_1 m_2 / (m_1 + m_2) × (V_1 - V_2)^2 = ?? J
> Share of the kinetic energy lost
η_T = ΔT / T_b = ??
> Impulse on body 2 (equal and opposite on body 1)
J = m_2 (v_2 - V_2) = ?? N·s
```

### Checks

Body 1 must be catching body 2, and the coefficient of restitution lies between 0 and 1.

```calc
@test V_1 > V_2
```

```calc
@test 0 ≤ e ≤ 1
```
