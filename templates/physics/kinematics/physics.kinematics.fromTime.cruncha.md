+++
cruncha-template = 1
id = "physics.kinematics.fromTime"
version = "1.0.0"
title = "Constant acceleration: velocity and distance after a time"
summary = "Straight-line motion under constant acceleration: the final velocity, the displacement and the average velocity after a given time."
category = ["Physics", "Kinematics"]
tags = ["kinematics", "constant acceleration", "velocity", "displacement", "braking", "suvat"]
from = "lacewing:physics.kinematics"
license = "GPL-3.0-or-later"
inputs = ["u", "a", "t"]
outputs = ["v", "s", "v_avg"]

[[references]]
work = "OpenStax University Physics Volume 1"
edition = "2016"
locator = "Sec. 3.4, Motion with Constant Acceleration (Eqs. 3.12, 3.13); Example 3.7"
url = "https://openstax.org/books/university-physics-volume-1/pages/3-4-motion-with-constant-acceleration"

[[tests]]
name = "An airplane landing: 70 m/s, braking at 1.50 m/s² for 40 s"
source = "Lacewing Lab's opening example, after OpenStax University Physics Vol. 1 Example 3.7; v and s by hand"
inputs = { u = "70 m/s", a = "-1.5 m/s^2", t = "40 s" }
expect = { v = "10 m/s", s = "1600 m", v_avg = "40 m/s" }

[[tests]]
name = "Dropped from rest for 2 s at standard gravity"
source = "By hand: v = 9.80665 × 2, s = 9.80665 × 2² / 2"
inputs = { u = "0 m/s", a = "9.80665 m/s^2", t = "2 s" }
expect = { v = "19.6133 m/s", s = "19.6133 m" }
+++

## Constant acceleration: from the time

Straight-line motion under an acceleration that stays constant for the whole time. Air resistance and anything else that changes the acceleration are left out.

Signs give the direction: choose a positive direction and keep to it. Braking is a negative acceleration against a positive initial velocity.

### Motion

```calc {description="Initial velocity, signed"}
u = 70 m/s_
```

```calc {description="Constant acceleration, signed like u"}
a = -1.5 m/s^2_
```

```calc {description="Time the acceleration acts"}
t = 40 s_
```

### Results

```calc {description="Final velocity"}
v = u + a t = ?? m/s
```

```calc {description="Displacement after the time t"}
s = u t + a t^2 / 2 = ?? m
```

The average velocity is the mean of the two ends only because the acceleration is constant.

```calc {description="Average velocity"}
v_avg = (u + v) / 2 = ?? m/s
```
