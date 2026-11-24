+++
cruncha-template = 1
id = "physics.kinematics.fromDistance"
version = "1.0.0"
title = "Constant acceleration: acceleration and time over a distance"
summary = "Straight-line motion under constant acceleration: the acceleration and time that take a body from one velocity to another over a given distance."
category = ["Physics", "Kinematics"]
tags = ["kinematics", "constant acceleration", "velocity", "displacement", "stopping distance", "suvat"]
from = "lacewing:physics.kinematics"
license = "GPL-3.0-or-later"
inputs = ["u", "v", "s"]
outputs = ["a", "t", "v_avg"]

[[references]]
work = "OpenStax University Physics Volume 1"
edition = "2016"
locator = "Sec. 3.4, Motion with Constant Acceleration (Eqs. 3.14, 3.16)"
url = "https://openstax.org/books/university-physics-volume-1/pages/3-4-motion-with-constant-acceleration"

[[tests]]
name = "The landing run backwards: 70 to 10 m/s over 1600 m"
source = "OpenStax University Physics Vol. 1 Example 3.7's landing (70 m/s, −1.50 m/s², 40 s) solved for a and t; by hand"
inputs = { u = "70 m/s", v = "10 m/s", s = "1600 m" }
expect = { a = "-1.5 m/s^2", t = "40 s", v_avg = "40 m/s" }
checks = "pass"

[[tests]]
name = "Speeding up from 20 to 30 m/s over 100 m"
source = "By hand: a = (30² − 20²)/(2 × 100) = 2.5 m/s², t = 2 × 100/(20 + 30) = 4 s"
inputs = { u = "20 m/s", v = "30 m/s", s = "100 m" }
expect = { a = "2.5 m/s^2", t = "4 s" }
checks = "pass"

[[tests]]
name = "Velocities against the distance: no constant acceleration covers it"
source = "u + v has the opposite sign to s, so the time comes out negative and the check fails"
inputs = { u = "5 m/s", v = "-10 m/s", s = "100 m" }
expect = { t = "-40 s" }
checks = "fail"
+++

## Constant acceleration: from the distance

Straight-line motion under an acceleration that stays constant over the whole distance. Air resistance and anything else that changes the acceleration are left out.

Signs give the direction: choose a positive direction and keep to it. The displacement must not be zero, and u + v must have the same sign as s, or no constant acceleration takes the body from u to v over that distance.

### Motion

```calc {description="Initial velocity, signed"}
u = 70 m/s_
```

```calc {description="Final velocity, signed like u"}
v = 10 m/s_
```

```calc {description="Displacement from start to finish, signed"}
s = 1600 m_
```

### Results

```calc
> Constant acceleration, from v² = u² + 2as
a = (v^2 - u^2) / (2 s) = ?? m/s^2
> Average velocity
v_avg = (u + v) / 2 = ?? m/s
> Time taken, s divided by the average velocity
t = s / v_avg = ?? s
```

### Check

```calc
@test t > 0 's'
```
