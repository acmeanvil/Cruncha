+++
cruncha-template = 1
id = "mechanical.boltedJoint.nasaMargins"
version = "1.0.0"
title = "Fastener preload range and margins of safety (NASA-STD-5020B)"
summary = "The maximum and minimum preload of one fastener from its nominal preload, preload variation, installation tolerance, relaxation, creep and temperature, and its margins of safety under limit tension and shear: ultimate and yield (with separation-before-rupture decided by linear joint theory), separation, shear, tension-shear interaction and slip."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt", "fastener", "preload", "preload variation", "margin of safety", "NASA-STD-5020", "spaceflight", "separation", "slip", "interaction", "fitting factor", "bolted joint"]
from = "lacewing:mechanical.boltedJoint"
license = "GPL-3.0-or-later"
inputs = ["P_(p,nom)", "Γ_p", "c_tol", "r_rel", "P_pc", "P_(Δt,max)", "P_(Δt,min)", "n_fas", "P_tL", "P_sL", "C", "n_li", "P_(tu,allow)", "P_(ty,allow)", "P_(su,allow)", "FS_u", "FS_y", "FS_sep", "FF", "μ", "threads"]
outputs = ["P_(i,max)", "P_(i,min)", "P_(i,pat)", "P_(p,max)", "P_(p,min)", "P_(p,pat)", "nφ", "P_tu′", "P_ty′", "P_sep′", "MS_u", "MS_y", "MS_sep", "MS_s", "MS_int", "MS_slip"]

[[references]]
work = "NASA-STD-5020B, Requirements for Threaded Fastening Systems in Spaceflight Hardware"
edition = "2021"
locator = "Sec. 4.3 (Eqs. 1 to 5; Tables 1 and 3, relaxation and preload variation), Sec. 4.4 (Eqs. 6, 7, 10, 11 ultimate tension; 14 shear; 15 to 17 yield; 19 separation; 20, 22 interaction; 84 to 86 slip), Sec. 4.4.6 (friction coefficients), App. A.4 (load-introduction factor)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-7 (joint constant C, Eq. 8-24)"

[[tests]]
name = "Separation before rupture: 10 kN nominal, Γ = 25 %, ±5 %, four fasteners"
source = "Lacewing Lab bolted_joint_test (NASA preload range and margins): Pp,max = 1.05 × 1.25 × 10 000 = 13 125 N, Pi,min = 0.95 × 0.75 × 10 000 = 7125 N, Pi,pattern = 0.95 × (1 − 0.25/√4) × 10 000 = 8312.5 N, Pp,min = 0.95 × 7125 = 6768.75 N; P′sep = 13 125/0.8 = 16 406 N < P′tu = 84 375 N, so Eq. 6: MSu = 30 000/(1.15 × 1.4 × 2000) − 1 = 8.31677, MSsep = 6768.75/2300 − 1 = 1.94293"
inputs = { "P_(p,nom)" = "10000 N", "Γ_p" = "0.25", c_tol = "0.05", r_rel = "0.05", P_pc = "0 N", "P_(Δt,max)" = "0 N", "P_(Δt,min)" = "0 N", n_fas = "4", P_tL = "2000 N", P_sL = "500 N", C = "0.2", n_li = "1", "P_(tu,allow)" = "30000 N", "P_(ty,allow)" = "25000 N", "P_(su,allow)" = "20000 N", FS_u = "1.4", FS_y = "1.0", FS_sep = "1.0", FF = "1.15", "μ" = "0.2", threads = "0" }
expect = { "P_(p,max)" = "13125 N", "P_(i,min)" = "7125 N", "P_(i,pat)" = "8312.5 N", "P_(p,min)" = "6768.75 N", "P_sep′" = "16406.25 N", MS_u = "8.316770", MS_sep = "1.942935" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: the same joint's yield, shear, interaction and slip margins"
source = "By hand: P′ty = (25 000 − 13 125)/0.2 = 59 375 N > P′sep, so MSy = 25 000/(1.15 × 2000) − 1 = 9.869565; MSs = 20 000/(1.15 × 1.4 × 500) − 1 = 23.844720; Eq. 20, (k × 805/20 000)^2.5 + (k × 3220/30 000)^1.5 = 1 at k = 8.841416, MSint = 7.841416; Pp,pattern = 0.95 × 8312.5 = 7896.875 N, MSslip = 0.2 × 7896.875/(1.15 × (500 + 0.2 × 2000)) − 1 = 0.525966"
inputs = { "P_(p,nom)" = "10000 N", "Γ_p" = "0.25", c_tol = "0.05", r_rel = "0.05", P_pc = "0 N", "P_(Δt,max)" = "0 N", "P_(Δt,min)" = "0 N", n_fas = "4", P_tL = "2000 N", P_sL = "500 N", C = "0.2", n_li = "1", "P_(tu,allow)" = "30000 N", "P_(ty,allow)" = "25000 N", "P_(su,allow)" = "20000 N", FS_u = "1.4", FS_y = "1.0", FS_sep = "1.0", FF = "1.15", "μ" = "0.2", threads = "0" }
expect = { "P_ty′" = "59375 N", MS_y = "9.869565", MS_s = "23.844720", MS_int = "7.841416", "P_(p,pat)" = "7896.875 N", MS_slip = "0.525966" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: rupture before separation (nφ = 0.6), threads in the shear plane"
source = "By hand: P′tu = (30 000 − 13 125)/0.6 = 28 125 N < P′sep = 13 125/0.4 = 32 812.5 N, so Eq. 7: MSu = 28 125/(1.15 × 1.4 × 2000) − 1 = 7.734472; P′ty = 19 791.67 N, MSy = 19 791.67/2300 − 1 = 7.605072; Eq. 22, (k × 805/20 000)^1.2 + (k × 3220/30 000)^2 = 1 at k = 8.026846"
inputs = { "P_(p,nom)" = "10000 N", "Γ_p" = "0.25", c_tol = "0.05", r_rel = "0.05", P_pc = "0 N", "P_(Δt,max)" = "0 N", "P_(Δt,min)" = "0 N", n_fas = "4", P_tL = "2000 N", P_sL = "500 N", C = "0.6", n_li = "1", "P_(tu,allow)" = "30000 N", "P_(ty,allow)" = "25000 N", "P_(su,allow)" = "20000 N", FS_u = "1.4", FS_y = "1.0", FS_sep = "1.0", FF = "1.15", "μ" = "0.2", threads = "1" }
expect = { "P_tu′" = "28125 N", "P_sep′" = "32812.5 N", MS_u = "7.734472", MS_y = "7.605072", MS_int = "7.026846" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Too little preload for 2 kN of limit shear: the joint can slip"
source = "As the first case with the limit tension doubled: MSsep = 6768.75/(1.15 × 4000) − 1 = 0.471467; and with a 2 kN limit shear MSslip = 0.2 × 7896.875/(1.15 × 2800) − 1 = −0.509511"
inputs = { "P_(p,nom)" = "10000 N", "Γ_p" = "0.25", c_tol = "0.05", r_rel = "0.05", P_pc = "0 N", "P_(Δt,max)" = "0 N", "P_(Δt,min)" = "0 N", n_fas = "4", P_tL = "4000 N", P_sL = "2000 N", C = "0.2", n_li = "1", "P_(tu,allow)" = "30000 N", "P_(ty,allow)" = "25000 N", "P_(su,allow)" = "20000 N", FS_u = "1.4", FS_y = "1.0", FS_sep = "1.0", FF = "1.15", "μ" = "0.2", threads = "0" }
expect = { MS_sep = "0.471467", MS_slip = "-0.509511" }
tolerance = "1e-5"
checks = "fail"
+++

## Fastener preload range and margins of safety (NASA-STD-5020B)

Margins of safety of one fastener in a preloaded joint by NASA-STD-5020B, the NASA standard for threaded fastening systems in spaceflight hardware. Preload scatters about its nominal value, so the standard works with a maximum preload (for rupture and yield) and a minimum preload (for separation and slip). A margin of safety MS = allowable / (factored load) − 1 passes at zero or more.

The loads are **limit** loads on this fastener (the largest expected in service), tension along it and shear per shear plane. The program's factors of safety (from NASA-STD-5001 or the project's requirements) and the fitting factor multiply them. Bending of the fastener is left out of the interaction, and preload is left out at ultimate, as the standard allows for ductile fasteners. Fatigue, thread shear-out and the clamped parts are separate checks.

### Preload

The nominal preload is what the tightening is aimed at, from the torque and nut factor or measured. The preload variation Γ_p depends on how the preload is set; Table 3 of the standard gives default values for torque control (lubricated or not), turn-of-nut and bolt stretch, and joints that must not separate need it from torque-tension tests. The installation tolerance c_tol is the ± tolerance on the torque or preload, as a fraction. Short-term relaxation r_rel is a fraction of the minimum initial preload (Table 1 gives a value for all-metal joints).

```calc {description="Nominal initial preload"}
P_(p,nom) = 10000 N_
```

```calc {description="Preload variation Γ, as a fraction (Table 3, or torque-tension tests)"}
Γ_p = 0.25_
```

```calc {description="Installation tolerance on torque or preload (± fraction)"}
c_tol = 0.05_
```

```calc {description="Short-term relaxation, fraction of the minimum initial preload (Table 1)"}
r_rel = 0.05_
```

```calc {description="Preload lost to creep"}
P_pc = 0 N_
```

```calc {description="Largest preload increase from temperature"}
P_(Δt,max) = 0 N_
```

```calc {description="Largest preload decrease from temperature"}
P_(Δt,min) = 0 N_
```

```calc {description="Fasteners sharing the load in the joint (for the pattern minimum preload)"}
n_fas = 4_
```

```calc
> Maximum initial preload, Eq. 3
P_(i,max) = (1 + c_tol) (1 + Γ_p) P_(p,nom) = ?? N
> Minimum initial preload, Eq. 4 (separation-critical joints, fatigue)
P_(i,min) = (1 - c_tol) (1 - Γ_p) P_(p,nom) = ?? N
> Minimum initial preload averaged over the pattern, Eq. 5 (slip; joints that are not separation-critical)
P_(i,pat) = (1 - c_tol) (1 - Γ_p / sqrt(n_fas)) P_(p,nom) = ?? N
> Maximum preload, Eq. 1
P_(p,max) = P_(i,max) + P_(Δt,max) = ?? N
> Minimum preload, Eq. 2
P_(p,min) = P_(i,min) (1 - r_rel) - P_pc - P_(Δt,min) = ?? N
> Minimum pattern preload in service
P_(p,pat) = P_(i,pat) (1 - r_rel) - P_pc - P_(Δt,min) = ?? N
```

### Loads, allowables and factors

The load-introduction factor n_li says where the external load enters the clamped parts: 1 at the head and nut faces, which is conservative (App. A.4); C is the joint constant (Shigley Eq. 8-24, or the bolted joint in tension template). The allowables are the fastener's ultimate and yield tensile loads and its ultimate shear load on the area in the shear plane. The faying surfaces' friction μ for slip is limited by Sec. 4.4.6 (lower for surfaces that are not clean bare metal). Enter threads = 1 when the threads are in the shear plane (Eq. 22), 0 when the shank is (Eq. 20).

```calc {description="Limit tensile load on the fastener"}
P_tL = 2000 N_
```

```calc {description="Limit shear load on the fastener, per shear plane"}
P_sL = 500 N_
```

```calc {description="Joint constant (stiffness ratio)"}
C = 0.2_
```

```calc {description="Load-introduction factor (1 at the head and nut faces)"}
n_li = 1_
```

```calc {description="Ultimate tensile load allowable of the fastener"}
P_(tu,allow) = 30000 N_
```

```calc {description="Yield tensile load allowable of the fastener"}
P_(ty,allow) = 25000 N_
```

```calc {description="Ultimate shear load allowable, per shear plane"}
P_(su,allow) = 20000 N_
```

```calc {description="Ultimate factor of safety"}
FS_u = 1.4_
```

```calc {description="Yield factor of safety"}
FS_y = 1.0_
```

```calc {description="Separation factor of safety"}
FS_sep = 1.0_
```

```calc {description="Fitting factor"}
FF = 1.15_
```

```calc {description="Friction coefficient of the faying surfaces (Sec. 4.4.6)"}
μ = 0.2_
```

```calc {description="1 if the threads are in the shear plane, 0 if the shank is"}
threads = 0_
```

### Tension: rupture, yield and separation

By linear joint theory the fastener takes nφ of the external load while the joint is closed and all of it once the joint has separated. Eqs. 10 and 11 find the external load at which the fastener would rupture with the joint closed, and the load at which the joint separates; whichever comes first decides whether the margin is taken on the fastener's allowable itself (separation first, Eq. 6) or on the external load that ruptures it (Eq. 7). Yield works the same way (Eqs. 15 to 17).

```calc
> Fraction of the external load the fastener takes, joint closed
nφ = n_li C = ??
> External load that ruptures the fastener with the joint closed, Eq. 10
P_tu′ = (P_(tu,allow) - P_(p,max)) / nφ = ?? N
> External load that yields it with the joint closed
P_ty′ = (P_(ty,allow) - P_(p,max)) / nφ = ?? N
> External load that separates the joint at the maximum preload, Eq. 11
P_sep′ = P_(p,max) / (1 - nφ) = ?? N
```

```calc {description="Ultimate tension margin, Eq. 6 (separation first) or Eq. 7"}
MS_u = { P_(tu,allow) if P_sep′ < P_tu′; P_tu′ otherwise } / (FF FS_u P_tL) - 1 = ??
```

```calc {description="Yield tension margin, Eqs. 15 to 17"}
MS_y = { P_(ty,allow) if P_sep′ < P_ty′; P_ty′ otherwise } / (FF FS_y P_tL) - 1 = ??
```

```calc {description="Separation margin at the minimum preload, Eq. 19"}
MS_sep = P_(p,min) / (FF FS_sep P_tL) - 1 = ??
```

### Shear and interaction

The interaction margin is the factor on both ultimate loads together at which the interaction curve is reached, less one: (P_s/P_su)^2.5 + (P_t/P_tu)^1.5 = 1 with the shank in the shear plane (Eq. 20), (P_s/P_su)^1.2 + (P_t/P_tu)^2 = 1 with threads (Eq. 22). It is found by bisection.

```calc {description="Ultimate shear margin, Eq. 14"}
MS_s = P_(su,allow) / (FF FS_u P_sL) - 1 = ??
```

```calc
function interactionMargin(ps, pt, psa, pta, xs, xt)
    lo = 0
    hi = 1000
    for i in 1:80
        k = (lo + hi) / 2
        if (k ps / psa)^xs + (k pt / pta)^xt < 1
            lo = k
        else
            hi = k
        end
    end
    return lo - 1
end
```

```calc
> Exponents of the interaction curve, Eq. 20 or Eq. 22
x_s = { 1.2 if threads ≥ 1; 2.5 otherwise } = ??
x_t = { 2 if threads ≥ 1; 1.5 otherwise } = ??
> Tension-shear interaction margin at ultimate, bending and preload omitted
MS_int = interactionMargin(FF FS_u P_sL, FF FS_u P_tL, P_(su,allow), P_(tu,allow), x_s, x_t) = ??
```

### Slip

Friction from the pattern's minimum preload must carry the limit shear, with the limit tension's share taken off the clamping (Eqs. 84 to 86, per fastener). A negative margin means the joint can slip and put the fasteners in bearing.

```calc {description="Slip margin per fastener, Eqs. 84 to 86"}
MS_slip = μ P_(p,pat) / (FF FS_y (P_sL + μ P_tL)) - 1 = ??
```

### Checks

```calc
@test MS_u ≥ 0
```

```calc
@test MS_y ≥ 0
```

```calc
@test MS_sep ≥ 0
```

```calc
@test MS_s ≥ 0
```

```calc
@test MS_int ≥ 0
```

```calc
@test MS_slip ≥ 0
```
