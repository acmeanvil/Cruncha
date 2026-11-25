+++
cruncha-template = 1
id = "mechanical.flywheels.punchPress"
version = "1.0.0"
title = "Punch-press flywheel on an induction motor (Shigley)"
summary = "The flywheel of a punch press driven by an induction motor with a linear torque-speed curve: the motor's rated torque and line, the motor torque and speed at the end of the punch from Shigley's Eq. 16-68, the inertia that lets the motor recover rated speed in the time left in the cycle, the resulting coefficient of speed fluctuation and the energy the flywheel gives up."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["flywheel", "punch press", "induction motor", "torque-speed curve", "speed fluctuation", "inertia", "Shigley"]
from = "lacewing:mechanical.flywheels"
license = "GPL-3.0-or-later"
inputs = ["P_r", "n_r", "n_s", "T_L", "t_p", "t_r"]
outputs = ["T_r", "a", "b", "T_2", "n_2", "I", "C_s", "E_p"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-12: Fig. 16-29, Eqs. 16-62 and 16-65 to 16-69 (pp. 861-863)"

[[tests]]
name = "Shigley's 3 hp, 1125 rpm motor: the torque line"
source = "Lacewing Lab brakes_test, after Shigley 10th pp. 862-863: rated 168.1 lbf·in, ωr 117.81 and ωs 125.66 rad/s give a = −21.41 lbf·in·s/rad and b = 2690.9 lbf·in; the punch (ten times Tr for 0.5 s, 1.5 s to recover) is illustrative"
inputs = { P_r = "3 hp", n_r = "117.81 rad/s", n_s = "125.66 rad/s", T_L = "1681 lbf·in", t_p = "0.5 s", t_r = "1.5 s" }
expect = { T_r = "168.1 lbf·in", a = "-21.41 lbf·in·s", b = "2690.9 lbf·in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Hand check: 1125/1200 rpm motor, punch ten times Tr for 0.5 s, 1.5 s to recover"
source = "By hand (bisection of Eq. 16-68, which the Lacewing Lab brakes_test verifies by integrating the motion): Tr = 3 hp/(1125 rpm) = 18.989091 N·m, a = −2.4177662 N·m·s, b = 303.82546 N·m, T2 = 86.987751 N·m (4.5809 Tr), n2 = 856.43004 rpm, I = 2.4177662 × 1.5/ln(4.5809328) = 2.3829706 kg·m², Cs = 0.27108700, E = 6953.1581 J"
inputs = { P_r = "3 hp", n_r = "1125 rpm", n_s = "1200 rpm", T_L = "189.89091 N·m", t_p = "0.5 s", t_r = "1.5 s" }
expect = { T_r = "18.989091 N·m", a = "-2.4177662 N·m·s", b = "303.82546 N·m", T_2 = "86.987751 N·m", n_2 = "856.43004 rpm", I = "2.3829706 kg·m²", C_s = "0.27108700", E_p = "6953.1581 J" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "A recovery longer than the motor needs: no flywheel solution"
source = "By hand: with tp = 0.1 s and 10 s to recover, tr exceeds (TL − Tr)/Tr × tp = 0.9 s, so the motor regains rated speed whatever the flywheel and Eq. 16-68 has no root above Tr"
inputs = { P_r = "3 hp", n_r = "1125 rpm", n_s = "1200 rpm", T_L = "189.89091 N·m", t_p = "0.1 s", t_r = "10 s" }
expect = { T_r = "18.989091 N·m" }
tolerance = "1e-6"
checks = "fail"
+++

## Punch-press flywheel on an induction motor

A punch press takes a large torque T_L for a short time t_p and almost none for the rest of the cycle; a flywheel carries the punch and the motor brings it back to speed in the time left, t_r (Shigley Sec. 16-12, Fig. 16-29). The induction motor's torque is taken as falling linearly from its rated torque T_r at the rated speed ω_r to nothing at the synchronous speed ω_s: T = a ω + b (Eq. 16-65), with a negative.

Integrating I dω/dt = a ω + b − T_L over the punch and I dω/dt = a ω + b over the recovery, and dividing the two times to eliminate I, gives Eq. 16-68 for T_2, the motor torque at the slowest speed at the end of the punch: T_2/T_r = ((T_L − T_r)/(T_L − T_2))^(t_r/t_p). It also has the trivial root T_2 = T_r; the one sought lies between T_r and T_L and is found here by bisection. The inertia then follows from the recovery. Shigley prints Eq. 16-69 as a t_r/ln(T_2/T_r), negative with his negative a; integrating from ω_2 up to ω_r gives its magnitude, which is used. The inertia is everything turning with the motor shaft.

If t_r is longer than (T_L − T_r)/T_r × t_p, the motor regains rated speed sooner whatever the flywheel and the equation has no root above T_r: size the flywheel on the speed drop allowed instead, with the energy-fluctuation template.

### Inputs

```calc {description="Rated power of the motor"}
P_r = 3 hp_
```

```calc {description="Rated (full-load) speed of the motor"}
n_r = 1125 rpm_
```

```calc {description="Synchronous speed of the motor"}
n_s = 1200 rpm_
```

```calc {description="Torque the punch demands at the motor shaft"}
T_L = 189.89091 N·m_
```

```calc {description="Duration of the punch"}
t_p = 0.5 s_
```

```calc {description="Time left in the cycle for the motor to recover rated speed"}
t_r = 1.5 s_
```

### Motor line

```calc
> Rated torque
T_r = P_r / n_r = ?? N·m
> Slope and intercept of the motor's torque line, Eq. 16-65
a = -T_r / (n_s - n_r) = ?? N·m·s
b = T_r n_s / (n_s - n_r) = ?? N·m
```

### Motor torque at the end of the punch

```calc
function punchTorque(T_r, T_L, k)
    lo = T_r (1 + 1e-9)
    hi = T_L (1 - 1e-12)
    for i in 1:200
        mid = (lo + hi) / 2
        if ln(mid / T_r) - k ln((T_L - T_r) / (T_L - mid)) > 0
            lo = mid
        else
            hi = mid
        end
    end
    return (lo + hi) / 2
end
```

```calc
> Motor torque at the slowest speed, the root of Eq. 16-68 above T_r
T_2 = punchTorque(T_r, T_L, t_r / t_p) = ?? N·m
> Slowest speed, from the motor line
n_2 = (T_2 - b) / a = ?? rpm
```

### Flywheel

```calc
> Inertia, Eq. 16-69 (its magnitude)
I = abs(a) t_r / ln(T_2 / T_r) = ?? kg·m²
> Coefficient of speed fluctuation that results, Eq. 16-62
C_s = (n_r - n_2) / ((n_r + n_2) / 2) = ??
> Energy the flywheel gives up during the punch
E_p = I (n_r^2 - n_2^2) / 2 = ?? J
```

### Checks

```calc
@test T_L > T_r
```

```calc
@test t_r < (T_L - T_r) / T_r t_p
```
