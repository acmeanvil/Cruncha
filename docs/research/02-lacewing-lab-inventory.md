<!-- Inventory of the Lacewing Lab calculators (~/Developer/Tension_Meter/App), gathered read-only on 2026-11-20 for ADR-0014 (calculation templates). Counts checked against registry.dart ids and help files: 286 each. -->

# Lacewing Lab: inventory of every calculator

## Summary
- **The source of truth is the code.** `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/lib/calculators/registry.dart` registers **286 calculators**. All 286 sit in the `catalog` tree, and all 286 have a help file. No topic is still marked "Coming later": every topic opens a built calculator.
- **Top-level sections in the code:** Materials, Geometry, Mechanical engineering, Machining, Structural engineering, Thermal engineering, Electrical engineering, Mathematics, Physics, Unit converter.
- **Not in the code:** there is no "Science" section. "Statics" was taken out of the catalog on purpose (CLAUDE.md line 219: "Stress and failure covers it").
- **Sections the outline lacks:** Materials, Machining (as its own section) and Thermal engineering.
- **How this was gathered:** read-only. I did not run the app or any build. Inputs, outputs and sources below come from each calculator's help file. Formulas come from the engine code and the help text.

## Path bases (every relative path below sits under one of these)
- **S** = `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/lib/calculators/` (screens)
- **E** = `/Users/admin/Developer/Tension_Meter/App/packages/engineering_core/lib/` (maths, pure Dart)
- **H** = `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/help/calculators/<id>.md` (one per calculator)
- **C** = `/Users/admin/Developer/Tension_Meter/App/packages/cicada_core/lib/` (materials, units)
- **U** = `/Users/admin/Developer/Tension_Meter/App/packages/cicada_ui/lib/ui/` (shared pages)
- Design notes, with per-calculator equations, tables, test examples and book slips: `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/CLAUDE.md` (1,044 lines; "State of play").
- Macro and registry design: `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/docs/MACROS.md`.

## 1. Tech stack and how calculators are structured (for a converter)

**Stack**
- Flutter/Dart workspace monorepo: the app is `App/apps/lacewing_lab`, version 0.1.0+1.
- Riverpod 3 for state.
- Three shared packages: `engineering_core` (all the maths, pure Dart, book-checked tests), `cicada_core` (materials, Quantities/units, cable catalog) and `cicada_ui` (calculator page template, material picker, charts).
- Embedded CPython 3.14 through `serious_python`, used for the RPN calculator and user macros:
  - Python package: `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/python/lacewing/` (`rpl.py`, `worksheet.py`, `wsmath.py`).
  - Python libraries: SymPy, pint.
- Other packages: pdf (reports), flutter_math_fork, archive/xml (xlsx import), share_plus.
- Only macOS has been built.
- `/Users/admin/Developer/Tension_Meter/App/lib` is the separate Cicada tension-meter app (BLE, spectrum analysis). It is not Lacewing Lab.
- `V2_ESP32/` holds firmware and a narrative text; `duckdb/macos` holds only libraries. Neither has calculators.

**How a calculator is registered**
- A calculator is `Calculator(id, title, summary, icon, page)` in the list `calculators` in registry.dart (lines 184-2366).
- It is placed by `CatalogNode(title, children | calculatorId)` in `const catalog` (lines 2494-2907).
- `standardGeometries`, `standardShapes` and `standardPackages` (lines 2440-2487) are groups reused inside Mechanical, Structural and Electrical.
- Ids look like `section.name` and are permanent.
- Adding a calculator means: a page, a registry entry, a catalog topic and a help file (`dart run tool/gen_help.dart` turns the Markdown into `lib/help/help_data.dart`).

**What a calculation returns**
- The engine returns `List<Formula>`, defined in E `geometry/section_shapes.dart` lines 41-51:
  ```dart
  Formula(this.symbol, this.expression, this.value, this.dim, {this.reference, this.note})
  ```
- The value is in SI. `Dim` (an enum, same file lines 6-26) says what kind of quantity it is, and `S/dims.dart` maps `Dim` to a display Quantity (SI or Imperial).
- Electrical and physics values have no unit system; they are shown with SI prefixes.
- Some engines (structural, thermal, math, cnc) return plain numbers or records instead. Their screens build `WorkingLine(symbol:, expression:, value:)`: about 238 occurrences under `apps/lacewing_lab/lib`.

**Declarative calculators (easiest to convert)**
- 20 screen files use the declarative `ModeCalculatorScreen` (S `mode_calculator.dart`): brakes/*, electrical field/inductance/induction/magnetic-circuit/practical/rlc, fastening/thread_screens, materials/corrosion, all physics/*, shafts/couplings, springs/other_spring_screens.
- Inputs are `CalcField(key, label, symbol, initialSI, kind: length|quantity|count|prefixed, unit, prefixes, scales, optional, quantity)`.
- A calculation is `CalcMode(label, fields, compute: Values → List<Formula>, figure, notes, presets)`.
- There are about 136 `CalcField(` and 231 `CalcMode(` uses. Example: `KinematicsScreen` in S `physics/physics_screens.dart` line 53.
- The other screens are hand-built `CalculatorPage`s (U `calculator/calculator_page.dart`) with the same card order: Inputs (with figure), Results, tables, Working (`symbol = formula = value`, reference), Notes.

**Function registry (already typed and unit-checked)**
- Lives in E `registry/function_registry.dart`, entries in `standard_functions.dart`.
- `EngFunction(id, title, summary, reference, inputs/outputs: Port(name, label, symbol, unit, kind), run, calculatorId)`.
- 24 ids are listed in `standard_functions.dart`; MACROS.md and CLAUDE.md say 26. The 24:
  - Structural: `structural.aisc360.compression/flexure/shear`, `structural.aisc370.compression`, `structural.adm.compression`, `structural.columns.euler/critical/alignmentK`, `structural.tensile.tensionFromFrequency/frequency`, `structural.asce7.velocityPressure/flatRoofSnow/seismicCs`, `structural.connections.boltShear/filletWeld/plateBuckling`.
  - Fluids: `fluids.statics.pressureAtDepth`, `fluids.pipes.frictionFactor`, `fluids.drag.force`.
  - Mechanics: `mechanics.catenary.ends`, `mechanics.capstan.tension`.
  - Thermal: `thermal.if97.pt/ph/saturation`.

**Help files (consistent and machine-readable)**
- H has 286 files with the headings `# Title`, `## How to use`, `## Inputs` (one `### Name` each, then `**symbol**: description`), `## Results` (same format, often with the formula in bold), `## Method`, `## Sources`.
- About 2,300 single-line `Formula('sym','expr'…)` strings can be pulled with:
  `grep -rhoE "Formula\(\s*'[^']{1,20}',\s*'[^']{1,120}'" --include='*.dart' /Users/admin/Developer/Tension_Meter/App/packages/engineering_core/lib`
- Formula counts per engine directory: physics 707, electrical 377, pressure 167, stress 155, springs 153, gears 145, brakes 132, fastening 129, bearings 107, drives 99, geometry 77, shafts 77, welding 66, materials 57, gdt 52, dynamics 47, machining 38, beams 22, fits 22, strain 13, fluids 7, tribology 5.

**Shared lookup data (generated by** `/Users/admin/Developer/Tension_Meter/App/scripts/gen_*.py`**)**
- Material library, 590+ materials: C `profiles/material_data.dart`, `material_details_data.dart`.
- Structural cable catalog: C `profiles/cable_catalog_data.dart`.
- Units: C `units/units.dart` (Quantities).

---

## 2. Calculators by heading
Each entry gives: name shown in the app (id) — screen (S) → engine (E). I = inputs (symbols), O = outputs, F = formulas/method, R = codes and references.

### MATERIALS (10; not in the outline)
- **Material library (materials.library)** — U `materials/materials_screen.dart`; data C `profiles/material_data.dart`, `material_details_data.dart`.
  - I: material type/group/name, ρ, E, σy/σu, strain.
  - O: G, K, ν, stress-strain point, tangent and secant modulus.
  - R: ASME VIII-2 Annex 3-D (stress-strain model), II-D Table PRD (ν).
- **Ramberg-Osgood curve (materials.rambergOsgood)** — U `materials/ramberg_osgood_screen.dart`.
  - I: E, σy (0.2%), n or a fit point.
  - O: n, α, K′, engineering and true curves, Et, Es; exports Ansys tables.
- **Periodic table (materials.periodicTable)** — S `materials/periodic_table_screen.dart` → E `chemistry/elements.dart` (data `elements_data.dart`).
  - I: element, a formula.
  - O: Z, atomic weight, properties, molar mass and composition of a formula.
  - R: PubChem, CIAAW 2024.
- **Material selection (performance indices) (materials.selection)** — S `materials/selection_screen.dart` → E `materials/selection.dart`.
  - I: component/objective (13 indices), type/group filters, minimum σy, minimum E, maximum ρ.
  - O: ranked materials, best index, log-log chart.
  - R: Ashby & Jones EM1 Chs. 7, 12, 16.
- **Composites and foams (materials.composites)** — S `materials/composites_screen.dart` → E `materials/composites.dart`.
  - I: Ef, ρf, σf, Em, ρm, σym, Vf, fibre d and l, orientation factor, particle properties, foam data.
  - O: Ec∥, Ec⊥, ρc, σTS, Vfcrit, E/ρ, σ/ρ, critical fibre length 2xc, fibre efficiency, foam properties.
  - R: Ashby & Jones EM2 Chs. 25-26, Eqs. 25.1-25.16, 26.1-26.5.
- **Creep rate and creep life (materials.creepRate)** — S `materials/creep_rate_screen.dart` → E `materials/creep_rate.dart`, `diffusion.dart`.
  - I: σ, T, n, Q, A, reference test point, life t, εlim, Monkman-Grant C, TM, second condition.
  - O: ε̇ss, ε over the life, stress for a strain limit, time to the limit, tf, bolt relaxation, T/TM.
  - F: ε̇ss = Aσⁿexp(−Q/RT); ε̇ss·tf = C.
  - R: EM1 Chs. 17-19, Eqs. 17.1-17.8, Table 17.1.
- **Diffusion (materials.diffusion)** — S `materials/diffusion_screen.dart` → E `materials/diffusion.dart`.
  - I: D0, Q (or EM1 Tables 18.1/18.2), TM, T, t, x, Cs, C0, C, Δc, wall thickness.
  - O: D, √(Dt), x²/D, C(x,t) (erf profile), case depth, time, flux J.
  - F: D = D0·exp(−Q/RT); Fick's first law.
  - R: EM1 Ch. 18.
- **Brittle fracture statistics (Weibull) (materials.weibull)** — S `materials/weibull_screen.dart` → E `materials/weibull.dart`.
  - I: ceramic (EM2 Table 15.7), test strength, survival probability, volumes, m, failure probability, safety factor, static-fatigue exponent, times.
  - O: design stress, σ0, V/V0, survival, m and σ0 fitted from tests (maximum likelihood), thermal shock.
  - R: EM2 Table 15.7, Chs. 17, 18, 20.
- **Oxidation and corrosion (materials.corrosion)** — S `materials/corrosion_screen.dart` → E `materials/corrosion.dart`.
  - I: x1, t1, t, xmax, Q, T1, T2, Ap, AL, Qp, QL, galvanic areas.
  - O: x, tmax, Kp, kL, mass gain to metal lost, Voxide/Vmetal, ΔE, corrosion rate, Ac/Aa, ia.
  - F: parabolic/linear kinetics with Arrhenius temperature dependence.
  - R: EM1 Chs. 21-24.
- **Pitting resistance (PREN) (materials.pitting)** — S `materials/pitting_screen.dart` → E `materials/pitting.dart`.
  - I: Cr, Mo, W, N, kN (16/22/30), typical-composition presets.
  - O: PREN, the share of each element, nearest grades with CPT/CCT.
  - F: PREN = Cr + 3.3(Mo + 0.5W) + kN·N.
  - R: Outokumpu Handbook Table 6:1, Forta Table 1, IMOA duplex guide.

### GEOMETRY (6)
**2D geometry > Properties of areas** (4 calculators on one screen)
- Screen: S `geometry/section_calculator_screen.dart` with `SectionFocus.area | secondMoment | sectionModulus | stiffness`.
- Engine: E `geometry/section_shapes.dart`, `plane_section.dart`, `section_results.dart`, `section_stiffness.dart`.
- Nine shapes, with inputs b, h, t, d, D, bf, tf, tw, legs.
- **Area (geometry.area)** — O: A, perimeter, x̄, ȳ.
- **Second moment of inertia (geometry.secondMoment)** — O: Ix, Iy, J, Ip, rx, ry, Ixy, Iu/Iv, θ, rv.
- **Section modulus (geometry.sectionModulus)** — O: Sx, Sy, Zx, Zy, shape factors, plastic neutral axes.
- **Stiffness of cross sections (geometry.sectionStiffness)** — I adds E, G (material), L. O: EA, EIx, EIy, EIv, GJ, member spring rates.
- R for all four: Roark 7th Table A.1 (case cited on each formula), Table 10.1 (J), Table 8.1 cases 1a, 1b, 1d, 1e (spring rates).

**3D geometry**
- **Properties of solids (geometry.solids)** — S `geometry/solid_properties_screen.dart` → E `geometry/solids.dart`.
  - 16 solids: sphere, hollow sphere, hemisphere, spherical segment, cylinder, tube, cone, frustum, hollow frustum, block, pyramid, pyramid frustum, torus, ellipsoid, paraboloid, prism of any 2D section.
  - O: M, V, surface, centroid, Ixc/Iyc/Izc, k, about the base and a parallel axis, principal moments.
  - F: hemisphere Iz = 83/320·Mr²; ellipsoid surface exact by DLMF §19.33.2 with Carlson RF/RD.
  - R: Blevins (2015) Table 1.6; Machinery's 26th pp. 67-71, 140-141.
- **Composite solids (geometry.compositeSolids)** — S `geometry/composite_solids_screen.dart` → E `geometry/solids.dart`.
  - I: parts added or taken away, origin, axis, polar copies, one density.
  - O: M, V, centroid, Ixx/Iyy/Izz, products, principal I1-I3 (Jacobi).
  - R: Blevins Table 1.6 cases 23-24.

### MECHANICAL ENGINEERING (101)
**Stress and failure (8)** — S `stress/` → E `stress/`
- **Von Mises and static failure (mechanical.staticFailure)** — `static_failure_screen.dart` → `static_failure.dart`, `math/symmetric3.dart`.
  - I: σx, σy, σz, τxy, τyz, τzx, Syt, Syc, Sut, Suc.
  - O: σ1 ≥ σ2 ≥ σ3, τmax, σ′ (von Mises), τoct; n by MSS, DE, ductile and brittle Coulomb-Mohr, modified Mohr; Mohr's circles.
  - R: Shigley 10th Eqs. 3-13, 3-15, 3-16, 5-3, 5-12…, Fig. 5-21.
- **Fatigue (mechanical.fatigue)** — `fatigue_screen.dart` → `fatigue.dart`, `juvinall_fatigue.dart`.
  - I: Sut, Sy, S′e, Marin factors, Kt and q, stresses, duty-cycle rows.
  - O: Se, σ′a, σ′m, nf by Goodman/Gerber/ASME-elliptic/Soderberg/Smith-Dolan, Langer ny, life, Miner D.
  - F: S′e (Eq. 6-8); kb = (d/7.62)^−0.107; Neuber q (Eqs. 6-34/35); S-N line (Eqs. 6-10 to 6-17).
  - R: Shigley ch. 6; Juvinall ch. 8.
- **Stress concentration factors (mechanical.stressConcentration)** — → `stress_concentration.dart`.
  - I: detail (shoulder, groove, notch, hole), load, D, d, r, t, load value.
  - O: Kt/Kts, σnom, σmax, q, Kf.
  - R: Peterson 3rd ed. chart fits (Charts 2.4, 2.19…).
- **Curved beams (mechanical.curvedBeams)** — → `curved_beams.dart`.
  - I: section dimensions, M, P.
  - O: σi, σo, Ki, Ko, M·c/I, rn (exact integration), e, A.
  - R: Juvinall Sec. 4.6; Shigley Sec. 3-18.
- **Residual stresses from yielding (mechanical.residualStresses)** — → `residual_stress.dart`.
  - I: b, h or D, di, yield depth, M or T, Sy, Ssy, E, G.
  - O: residual stresses, yield and plastic moments/torques, curvature or twist.
  - R: Juvinall Secs. 4.14-4.15.
- **Fracture mechanics (mechanical.fractureMechanics)** — → `fracture.dart`, `crack_factor_data.dart`.
  - I: configuration, c, w, σ, Y, KIc, Sy, t, Paris constants, pressure-vessel data.
  - O: KI, KIc/KI, failure stress, critical crack, plane-strain thickness, Paris cycles, leak-before-break.
  - F: K = Yσ√(πc).
  - R: Juvinall Secs. 6.3-6.4, 8.16, Table 6.1.
- **Fracture toughness calculator (KIc) (mechanical.kIcSolver)** — `kic_solver_screen.dart` → `fracture.dart`.
  - Solves the same K = Yσ√(πc) for any one of K, flaw size, stress or KIc.
- **Reliability (stress-strength interference) (mechanical.reliability)** — → `reliability.dart`.
  - I: μS, σS, μs, σs, goal, coefficients of variation.
  - O: R, margin, design factor, normal probabilities.
  - R: Juvinall Secs. 6.13-6.15, App. H; Shigley.

**Strain measurement (2)** — S `strain/` → E `strain/`
- **Strain gauges (mechanical.strainGauges)** — → `strain_gauge.dart`.
  - I: arrangement (7 of TN-507 Table 1), wiring, GF, leads, excitation E, material (E, ν, α), section.
  - O: load P/M/T/F, ε, σ or τ, expected output, sensitivity.
  - F: Eo/E = a/(a+b) − d/(d+c).
  - R: Micro-Measurements TN-507, TN-504, TT-612.
- **Strain rosettes (mechanical.strainRosettes)** — → `rosette.dart`.
  - I: εa, εb, εc, θb, θc, E, ν.
  - O: ε1, ε2, angle, γmax, σ1, σ2, τmax, von Mises.
  - R: Juvinall §5.2-5.5.

**Fastening (12)** — S `fastening/`, `welding/` → E `fastening/`, `welding/`

*Threads*
- **Thread strength (mechanical.threads)** and **Thread strength, any size (mechanical.threadsAnySize)** — `thread_strength_screen.dart` (`arbitrary: true` for any size) → `thread_strength.dart`, `thread_standards.dart`.
  - Tables: `thread_tables.dart`, `metric_thread_tables.dart`, `din513_iso7_tables.dart`.
  - I: form, size (or d and pitch), class, Le, bore, materials, design factor, loads.
  - O: At, shear areas As and An, σ, τ, σvm, stripping shear, capacities, utilization.
  - R: ASME B1.1-2019 §5.8, B1.13M Tables 9-13 and App. B-2, ISO 965-1, B1.5 Acme, DIN 103 / ISO 2904, B1.9, DIN 513-3, B1.20.1 NPT, ISO 228-1, ISO 7-1; Machinery's Eqs. 5-6.
- **Power screws (mechanical.powerScrews)** — `thread_screens.dart` → `threads.dart`.
  - I: d, p, starts, F, f, fc, dc, nut length.
  - O: raise and lower torque, efficiency e, self-locking, body and thread stresses.
  - R: Shigley Eqs. 8-1 to 8-12, Tables 8-1, 8-2, 8-3.

*Torque, clamping and rivets*
- **Bolt torque (simple) (mechanical.boltTorqueSimple)** — `bolt_torque_simple_screen.dart` → `bolt_torque.dart`, `vdi2230.dart`.
  - I: d, P (coarse if blank), dW (1.5d if blank), dh (ISO 273), μG, μK, FM, or ν·Rp0.2.
  - O: MA, σM, FM, torque shares, nut factor K.
  - R: VDI 2230-1:2015 Eqs. 119-129 (Eq. 127), Eq. 144, Tables A1, A5.
- **Bolt torque and clamping (mechanical.boltedJoint)** — `bolted_joint_screen.dart` → `bolted_joint.dart`, `bolt_grades.dart`.
  - I: thread, grade, L, Eb, dw, cone angle, layers (t, E), loads.
  - O: Fi, joint constant C, np, n0, nf, NASA preload range and margins.
  - R: Shigley ch. 8 (Tables 8-1 to 8-17, Eqs. 8-13 to 8-47); NASA-STD-5020B Eqs. 1-36, 84-86.
- **Bolted joints, VDI 2230 (mechanical.vdi2230)** — `vdi2230_screen.dart` → `vdi2230.dart`.
  - I: thread, lK, l1, dT, db, ES, dW, dh, joint data, FA, FQ, embedding.
  - O: FMmax, FMzul, MA, Φ, σred,B, σa, pmax, SG, Table A7 estimate.
  - F: σASV = 0.85(150/d + 45).
  - R: VDI 2230-1:2015 steps R0-R13, Tables 2, 5, A1-A9.
- **Bolt groups (mechanical.boltGroups)** — `bolt_group_screen.dart` → `bolt_group.dart`.
  - I: bolt coordinates, d, Fx, Fy and their points, Mz, P, Mx, My, t, preload.
  - O: largest shear, tension and bearing per bolt, centroid, slip.
  - R: Shigley Sec. 8-12, Eqs. 8-53 to 8-57; NASA Eq. 86.
- **Riveted joints (mechanical.rivetedJoints)** — `riveted_joint_screen.dart` → `riveted_joints.dart`.
  - I: lap or butt, w, t, strap, d, dh, rivets, allowables.
  - O: capacity, efficiency, edge ratio, eccentric-group rivet load.
  - R: Shigley Sec. 8-12; Juvinall Sec. 11.

*Welds, adhesives and brazing*
- **Weld group (mechanical.weldGroup)** — `weld_group_screen.dart` → `weld_lines.dart`, `weld_design.dart`, `weld_codes.dart`.
  - I: outline (12, Schaum Fig. 25-7a), b, d, V, e, a, P, static or fatigue, material, code, filler.
  - O: required leg w, f per length, Lw, Zw, Jw, centroid, AWS minimum leg.
  - R: Schaum's Machine Design ch. 25; AWS D1.1 Table 2.3, AISC 360 J2.5, D1.5/AASHTO, ASME VIII-1 UW-18, D1.6, AISC 370, D1.2/ADM, D1.9.
- **Weld capacity (mechanical.weldCapacity)** — `weld_capacity_screen.dart` → `weld_design.dart`, `weld_fatigue.dart`.
  - I: fillet, butt or plug; w, L, t, s_t, e, D, code.
  - O: Fall, throat, AWS minimum leg, Gerber/Goodman fatigue n.
  - R: Shigley Sec. 9-7, Table 9-5; Juvinall; Schaum.
- **Intermittent welds (mechanical.intermittentWelds)** — `intermittent_welds_screen.dart`.
  - I: continuous leg w_c, leg used w_i.
  - O: ratio R, length/pitch to use.
  - R: Schaum ch. 25 problem 17.
- **Adhesive and brazed joints (mechanical.adhesiveJoints)** — `adhesive_joint_screen.dart` → `welding/adhesive_joints.dart`.
  - I: single, double or brazed lap; E, ν, t, G, Ea, bond line, overlap, ΔT.
  - O: average and peak shear (Volkersen, Hart-Smith, Goland-Reissner), peel, thermal shear.
  - R: Juvinall 11.7-11.8; Shigley.

**Springs (10)** — S `springs/coil_spring_screens.dart`, `springs/other_spring_screens.dart`, `fastening/disc_spring_screen.dart` → E `springs/coil_springs.dart`, `other_springs.dart`, `disc_spring.dart` (+ `disc_spring_tables.dart`)
- **Compression springs (mechanical.compressionSprings)**
  - I: wire (Shigley Table 10-4), Sut, G, E, ends (Table 10-1), round or rectangular wire, d, OD, Na, loads.
  - O: k, y, τ, n, nf, ns, Fs, Ls, L0, critical L0, fn, C.
  - F: Sut = A/d^m; Wahl factor; buckling Eq. 10-12; surge Eq. 10-25.
  - R: Shigley Ch. 10; Roark 10.6-1 to 10.6-6.
- **Extension springs (mechanical.extensionSprings)**
  - I: wire, d, OD, Nb, hook radii r1/r2, Fi, loads.
  - O: k, y, nA, n body, nB, nf, L0, τi, C.
  - F: Na = Nb + G/E; initial-tension band 33 500/exp(0.105C).
  - R: Shigley Sec. 10-11.
- **Torsion springs (mechanical.torsionSprings)**
  - I: wire, d, OD, Nb, end lengths l1/l2, pin Dp, moments.
  - O: k′, M yield, θ′, σ, Δ, nf.
  - R: Shigley Sec. 10-12, Eqs. 10-43 to 10-61.
- **Belleville springs (mechanical.discSprings)**
  - I: EN 16983 size or De, Di, t, l0, t′; E, ν; stacking (series × parallel); working points; friction; cycles.
  - O: F, Fc, σOM, stresses I-IV, rate, relaxation, test load, fatigue life.
  - R: DIN 2092 (Schnorr Formulas 1-26); EN 16983 Tables 2, 4-6, 10, 12, Figs. 3-7.
- **Wave springs (mechanical.waveSprings)**
  - I: OD/ID, t, N waves, Z turns, E, P, H, Sut.
  - O: b, Dm, f, k, S, working height, K.
  - R: Smalley published formulas.
- **Conical springs (mechanical.conicalSprings)**
  - I: d, D1, D2, Na, G, F, Ssy.
  - O: k (Shigley and Juvinall), y, KB, τ, n, solid height.
  - R: Shigley Prob. 10-29; Machinery's p. 327.
- **Flat springs (mechanical.flatSprings)**
  - I: L, b, t, E, P, Sy.
  - O: k, P, f, Sb, n, U.
  - R: Machinery's 26th Table 19 (4 cases).
- **Spiral springs (mechanical.spiralSprings)**
  - I: b, t, L, E, M, Sy.
  - O: k, θ, σ, n, U.
  - F: strip in uniform bending.
- **Leaf springs (mechanical.leafSprings)**
  - I: 2L, b, t, ng, nf, E, W, Sy, Wmin.
  - O: F, δ, k, σ graduated and full-length, nip C, NPTEL δ cross-check, Goodman/Gerber nf.
  - F: σ = 12FL/(bt²(3nf + 2ng)); δ = 12FL³/(Ebt³(3nf + 2ng)); C = 2FL³/(nEbt³).
  - R: Machinery's Table 19 cases 2-3; NPTEL Module 7.
- **Torsion bar springs (mechanical.torsionBars)**
  - I: d, di, side or b×c, leaves, l, G, T or θ or force on an arm R.
  - O: J, K, k, τmax, θmax, U, arm rate k/R².
  - R: Shigley Eqs. 3-35 to 3-41; Roark Table 10.1 cases 3-4.

**Pressure vessels (4)** — S `pressure/` → E `pressure/`
- **Cylinders, spheres and cones (mechanical.pressureVessels)** — → `pressure_vessels.dart`.
  - I: shape, thin/thick/auto, capped, Di, t, pi, po, cone α, l, material.
  - O: σ hoop, σ longitudinal, σ radial, von Mises, n yield, ΔD, Δl, R/t.
  - R: Roark Tables 13.1 (1b, 1c, 2a, 3a) and 13.5.
- **Pressure vessel heads (mechanical.pressureHeads)** — → `pressure_heads.dart`.
  - I: code, head type (hemispherical, ellipsoidal, F&D, torispherical, Klöpper, Korbbogen), D, D/2h, L, r, P, S, E, corrosion allowance.
  - O: t, t + c, MAWP, code limits, depth, capacity, mass, blank diameter.
  - R: ASME VIII-1 UG-32 and App. 1-4, VIII-2 4.3.5-4.3.7, EN 13445-3 7.4/7.5, DIN 28011/28013.
- **Rectangular plates under pressure (mechanical.rectangularPlates)** — → `rectangular_plates.dart`.
  - I: edges and load case, a, b, t, p0 or P, E, Sy.
  - O: w, σ, M1.
  - F: σ = 6M1/t²; Navier series for the simply supported uniform case.
  - R: Den Hartog Adv. Strength §19 Cases 18-25, Eq. 68.
- **Stiffened flat plates under pressure (mechanical.stiffenedPlates)** — → `stiffened_plates.dart`.
  - I: ends, edges, material, P, L, pitch p, t, corrosion allowance, S, Sy, stiffener section.
  - O: maximum pitch, required t, effective width, plate and stiffener stresses.
  - R: ASME VIII-1 UG-47, App. 13-8, Table 13-8(d)/(e).

**Bearings (7)** — S `bearings/`, `tribology/`, `fits/` → E `bearings/`, `tribology/wear.dart`, `fits/`
- **Rolling bearings (mechanical.rollingBearings)** — `rolling_bearings.dart`.
  - I: type, method, catalogue size, α, C, C0, e, Fr, Fa, speed, reliability, lubricant.
  - O: L10, P, Kr·L10, L50, R, C10 required, Lnm, s0, n·dm, κ, ν1, friction.
  - R: Shigley Ch. 11 (Tables 11-1 to 11-6, Eqs. 11-1 to 11-24); ISO 281 a1; ABMA 9/11; SKF/JTEKT.
- **Hertz contact stress and life (mechanical.hertzContact)** — `hertz_contact.dart`.
  - I: contact type, E1, E2, ν1, ν2, F, length, diameters or principal radii.
  - O: pmax, a, b, δ, stiffness, τmax, σ′, τ0, SC, n, life.
  - R: Shigley Sec. 3-19, Eqs. 3-68 to 3-77; NASA RP-1255 Ch. 19; RP-1105; Roark Sec. 14.1; ISO 76.
- **Wear (mechanical.wear)** — `tribology/wear.dart`.
  - I: K (Fig. 9.12), hardness H, F, speed, time, area, allowed depth, pin-on-disk data.
  - O: wear volume, depth, pressure, life, K from a test.
  - F: W = KFS/H.
  - R: Juvinall 9.9-9.12.
- **Bearing fits (mechanical.bearingFits)** — S `fits/bearing_fits_screen.dart` → `fits/bearing_fits.dart`, `iso_fits.dart`, data `fit_tables_data.dart`.
  - I: d, D, type, ABEC class, load, shaft and housing class.
  - O: recommended classes, fits, limits.
  - R: Machinery's (ABMA 20 Tables 1-5, ABMA 7 Tables 13-14).
- **Journal bearings (mechanical.journalBearings)** — `journal_bearings.dart`, `lubricants.dart`.
  - I: d, c, l, N, W, lubricant (SAE or ISO VG, SG, two-point viscosity).
  - O: h0, S, f, T, H, Q, Qs, ΔT, Tav, Tmax, Tb, pmax.
  - F: Raimondi-Boyd charts (Harnoy Table 8-1).
  - R: Shigley Ch. 12.
- **Boundary-lubricated bushings (mechanical.boundaryBearings)** — `bushings.dart`.
  - I: F, D, L, N, nd, material, K, f1, f2.
  - O: PV, P, Pmax, V, wear w, life, film temperature Tf, minimum L.
  - R: Shigley Sec. 12-15 (Tables 12-7 to 12-12); Juvinall.
- **Lubricant viscosity (mechanical.lubricantViscosity)** — `lubricants.dart`.
  - I: SAE or ISO VG grade, SG, two points, T, Saybolt seconds.
  - O: μ, ν, ρ, SUS conversions.
  - R: Shigley Eq. 12-3, Table 12-1; Seireg fits; SKF Diagram 13.

**Shafts (5)** — S `shafts/` → E `shafts/`
- **Shaft design (mechanical.shaftDesign)** — `shaft_design.dart`.
  - I: find n or d; d, n, di/d, Ma, Mm, Ta, Tm, Fa, Fm, Sut, Sy, Kt or notch geometry.
  - O: n by DE-Goodman, DE-Gerber, DE-ASME elliptic, DE-Soderberg; ny; σ′a, σ′m; Se; Kf, Kfs; twist; deflection and slope diameters.
  - R: Shigley 7-4, 7-5, Table 7-1; Juvinall.
- **Shaft critical speed (mechanical.shaftCriticalSpeed)** — `critical_speed.dart`.
  - I: l, d, E, ρ, n, masses and positions, δij.
  - O: exact lumped-mass value, Rayleigh, Dunkerley, shaft alone, ratio to operating speed.
  - R: Shigley Sec. 7-6, Eqs. 7-22 to 7-34.
- **Keys, pins and splines (mechanical.keysSplines)** — `keys.dart`, `splines.dart`.
  - I: d, T, n, key w, h, l, Sy of key/shaft/hub, Woodruff key, pins, spline form.
  - O: standard key, length needed, factors of safety, spline pitch diameter and stresses.
  - R: Shigley Sec. 7-7, Tables 7-5 to 7-8; ANSI B17.1; BS 4235 / DIN 6885; Dudley; SAE / ISO 14.
- **Interference fits (mechanical.interferenceFits)** — `interference_fit.dart` + `fits/iso_fits.dart`.
  - I: d, di, do, l, δmin, δmax (or an ISO fit), E, ν, α, Sy, f.
  - O: p, hub and shaft stresses, n, torque and axial capacity, pressing force, shrink temperature.
  - R: Shigley 3-16, 7-8.
- **Couplings and universal joints (mechanical.couplings)** — `couplings.dart`.
  - I: u, ω1, input angle a, T1, u1, u2, phase error φ, bolts n, db, Dc, t, Ssy.
  - O: ω2/ω1, ω2 maximum and minimum, θ2, α2, flange torque capacity.
  - F: Hooke's joint speed ratio.
  - R: Marks p. 8-36; Machinery's pp. 2327-2328; Juvinall.

**Fits and tolerances (2)**
- **Limits and fits (mechanical.fits)** — S `fits/limits_fits_screen.dart` → E `fits/iso_fits.dart`, `inch_fits.dart`, `fit_tables_data.dart`.
  - I: D, preferred fit or hole/shaft positions and grades, inch class.
  - O: maximum and minimum clearance, limits.
  - R: ISO 286 (BS 4500 tables), ANSI B4.2, ANSI B4.1 RC/LC/LT/LN/FN (Machinery's 26th).
- **General tolerances (ISO 2768) (mechanical.iso2768)** — S `gdt/iso2768_screen.dart` → E `gdt/iso2768.dart`.
  - I: designation, class, sizes.
  - O: linear, broken-edge, angular, straightness, flatness, perpendicularity, symmetry, run-out deviations.
  - R: ISO 2768-1:1989, ISO 2768-2.

**GD&T (6)** — S `gdt/` → E `gdt/` (`characteristics.dart`, `frame.dart`, `rules.dart`, `explain.dart`, `tolerance_calcs.dart`, `iso_material_requirements.dart`, `stack_up.dart`)
- **Feature control frame builder (mechanical.gdtFrame)** — builds and checks a frame; O: MMVS, LMVS, virtual and resultant conditions, allowed deviation. R: ASME Y14.5-2018, ISO 1101/5459/2692/8015.
- **GD&T symbol reference (mechanical.gdtSymbols)** — reference only, no calculation.
- **Bonus tolerance and virtual condition (mechanical.gdtBonus)** — I: LL, UL, t, MAX, actual size AS, VS. O: bonus, MMC/LMC, VC, RC, boundaries. R: Y14.5 5.9 to 5.9.7.
- **True position from measurements (mechanical.gdtPosition)** — I: basic and measured X/Y/Z, ±tolerances, Ø zone. F: position = 2√(dx² + dy²); O: pass/fail with bonus. R: Y14.5.1-2019 7.2.2.
- **Fastener hole tolerances (floating and fixed) (mechanical.gdtFasteners)** — I: H, F, T1, P, D. O: hole, positional tolerance, projected zone. R: Y14.5 App. B.
- **Tolerance stack-up (mechanical.toleranceStack)** — I: rows of nominal, ±, sign, ISO 2768 class. O: worst case, RSS, shares. R: Shigley Sec. 1-14.

**Clutches and brakes (6)** — S `brakes/` (ModeCalculator) → E `brakes/`
- **Disk clutches and brakes (mechanical.diskClutches)** — `disk_clutches.dart`.
  - I: ro, ri, f, pa, N, T, F, caliper pad data.
  - O: re, Fmax, T capacity, N required, optimum ri/ro, pad results.
  - R: Juvinall 18.2-18.3; Shigley.
- **Cone clutches and brakes (mechanical.coneClutches)** — I: ro, ri, α, f, pa, T, F. O: b, re, Fmax, T, F for T, α check. R: Juvinall 18.5.
- **Drum brakes (mechanical.drumBrakes)** — `drum_brakes.dart`.
  - Short shoe, long internal/external shoe, pivoted shoe.
  - I: F, arms, r, w, θ1, θ2, pa, f.
  - O: N, T, pin reactions Rx/Ry, self-locking, pV.
  - R: Shigley 16-2, 16-3 (Eqs. 16-1 to 16-18).
- **Band brakes (mechanical.bandBrakes)** — `band_brakes.dart`.
  - I: r, b, φ, f, arms a/c/s, pa, F.
  - O: P1/P2 = e^(fφ), P1, P2, T, F, self-locking.
  - R: Juvinall 18.9, Eqs. 18.24-18.28.
- **Brake energy and temperature rise (mechanical.brakeEnergy)** — `brake_energy.dart`.
  - I: I, n0, nf, T, t, m, cp, M, h, share, I1, I2.
  - O: E, t, P, ΔT, repeated-stop temperatures.
  - R: Juvinall 18.4, Table 18.3; Shigley.
- **Flywheels (mechanical.flywheels)** — `flywheels.dart`.
  - I: E2 − E1, Cs, n, ρ, ro, ri, ν, motor P, nr, ns, punch TL and times.
  - O: I, n1, n2, m, rim stresses σt and σr maximum.
  - R: Shigley 16-12, Eqs. 16-61 to 16-69, 3-55.

**Belts and chains (5)** — S `drives/` → E `drives/`
- **Flat belts (mechanical.flatBelts)** — I: belt (Table 17-2), crossed, Hnom, n, d, D, C, b, t, γ, f, allowable tension. O: nfs, f′, (F1)a, F2, Fi, Fc, T, bmin, φ, dip, shaft load. R: Shigley 17-2 (Eqs. 17-1 to 17-15).
- **V-belts (mechanical.vBelts)** — I: section A-E or 3V/5V/8V, Hnom, n, d, D, L, Nb, Ks. O: Nb, nfs, Ha, Hd, C, V, F1, F2, Fi, life in passes and hours. R: Shigley 17-3 (Tables 17-9 to 17-17); Juvinall.
- **Timing belts (mechanical.timingBelts)** — I: section MXL-XXH, service factor, H, n, N1, N2, C. O: width, rating, length, pitch diameters, teeth in mesh, V. R: Machinery's Tables 33-42 (ANSI/RMA IP-24).
- **Roller chains (mechanical.rollerChains)** — I: ANSI 25-240, strands, Hnom, n1, N1, N2, C, Ks, nd. O: Ha, n, lubrication, length, C, pitch diameters, chordal variation, F. R: Shigley 17-5 (Tables 17-19 to 17-23); ACA equations.
- **Wire rope (mechanical.wireRope)** — I: rope (Table 17-24), material, sheave, d, D, W, m, l, a, p/Su. O: nf, ns, Ft, Fu, Fb, Ff, p. R: Shigley 17-6.

**Gear design (6)** — S `gears/` → E `gears/` (`geometry.dart`, `rating.dart`, `rating_juvinall.dart`, `j_factor.dart` + `j_factor_data.dart`, `bevel.dart`, `worm.dart`, `gear_trains.dart`)
- **Spur and helical gear geometry (mechanical.gearGeometry)** — I: m or Pd, external/internal, tooth system, φn, ψ, NP, NG, profile shifts x1/x2, ha, hf. O: a, φw, mp, mG, Bt, mN, interference. R: Shigley ch. 13; Machinery's pp. 2015, 2028; AGMA 2002.
- **Spur and helical gear rating (mechanical.gearRating)** — I: m, φn, ψ, N, F, H, nP, J, materials, Q, factors. O: SF/SH for pinion and gear, Wt, capacity. R: AGMA 2001-D04 via Shigley ch. 14; Juvinall ch. 15.
- **Straight bevel gears (mechanical.bevelGears)** — O: rated power, SF/SH, σc, pitch angles γ/Γ, A0, addenda, forces. R: AGMA 2003-B97 (Shigley ch. 15, Eqs. 15-1 to 15-25).
- **Worm gears (mechanical.wormGears)** — I: px, NW, NG, d, φn, FG, nW, H0, Ka, nd. O: efficiency eW, HW, λ, C, stresses, sump temperature, Wr. R: Shigley Eqs. 15-26 to 15-64; AGMA 6034/6022.
- **Gear trains (mechanical.gearTrains)** — simple/compound, planetary, differential. O: train value e, speed ratio, torques, planet speed. R: Juvinall 15.13 (Willis' equation).
- **Tooth thickness measurement (mechanical.toothThickness)** — `GearGeometryScreen(measurement: true)`. O: chordal thickness, measurement over pins/balls, span. R: AGMA 2002-B88.

**Hydrodynamics (4)** — S `fluids/fluid_screens.dart` → E `fluids/fluids.dart`
- **Depth and pressure (fluids.depthPressure)**
  - I: liquid (Marks Table 3.3.1), T, ρ, g, p0, h, pg, gate H, b, L, d, θ.
  - O: ρgh, p0 + ρgh, water and mercury heads, gate force F = ρg·hc·A, centre of pressure hF = hc + IG·sin²θ/(hc·A).
  - R: Marks Sec. 3.3.
- **Buoyancy (fluids.buoyancy)**
  - I: ρf, L, B, D, m, KG, or V and ρb.
  - O: V = m/ρf, T = V/(L·B), KB = T/2, BM = I/V with I = L·B³/12, GM = KB + BM − KG, buoyant force ρf·g·V, apparent weight.
  - R: Marks 3.3, 11.3.
- **Fluid drag (fluids.drag)**
  - I: fluid, shape (Table 11.4.4), ρ, μ, V, d, l, A, ρb.
  - O: FD = CD·ρV²A/2, Re = ρVd/μ, ρV²/2, shedding f = S·V/d, terminal velocity.
  - Data: CD(Re) from Marks Fig. 11.4.23.
- **Pipe flow (fluids.pipeFlow)**
  - I: NPS (Crane B-16), roughness (A-23), d, L, Q, ρ, μ, Δz, η, K, gas p1/p2, M, T, fittings.
  - O: ΔP = K·ρv²/2, total pressure, v = Q/A, f (Colebrook or 64/Re), K, pump power ρgQH/η, mass flow w.
  - R: Crane TP-410 Eqs. 1-4 to 1-10, 3-1 to 3-4, A-23 to A-29.

**Dynamics > Vibrations (6)** — S `dynamics/` → E `dynamics/vibration.dart`
- **Single degree of freedom (dynamics.singleDof)** — I: m, k, c/cc or c, P0, ω. O: ωn = √(k/m), resonant magnification, x0/xst, transmissibility, x0. R: Den Hartog Vib. ch. II, Eqs. 12-36.
- **Vibration absorber (dynamics.absorber)** — I: M, K, m, P0, tuning f, c/cc. O: μ = m/M, k = m(fΩn)², optimum c, resonant amplitude, Lanchester damper. R: Eqs. 50, 57, 63-69b.
- **Natural frequencies (dynamics.naturalFrequencies)** — beams, bars, shafts, strings, rings, membranes, plates, springs with mass. I: E, G, ρ, d, l, r, t, T, μ. O: fundamental and modes. R: Den Hartog App. II.
- **Torsional vibration (dynamics.torsional)** — I: disk inertias I and shaft stiffnesses k. O: modes and mode shapes. F: Holzer's method. R: §40-41.
- **Critical speeds (dynamics.criticalSpeeds)** — I: L, d, E, ρ, n, e, masses at positions x. O: critical speeds, whirl radius. R: §46a, Eq. 149b.
- **Two-plane balancing (dynamics.balancing)** — I: as-found readings and two trial runs (complex values). O: corrections in planes I and II, influence numbers. R: §48, Eqs. 150a/b.

**Mechanics (4)** — S `dynamics/mechanics_screens.dart`, `dynamics/impact_loading_screen.dart` → E `dynamics/mechanics.dart`, `impact_loading.dart`
- **Rope friction on a drum (mechanics.capstan)** — I: T0 or T1, f, n turns. O: T1 = T0·e^(fφ), wrap needed. R: Den Hartog Mechanics §20i.
- **Impact (mechanics.impact)** — I: m1, m2, V1, V2, e. O: v1, v2 after impact, energy lost, impulse. R: §53 Eqs. 28-30.
- **Impact stresses and deflections (mechanical.impactLoading)** — I: struck structure, W, h, v, k, L, E, A, I, S, strength. O: impact factor, equivalent static force, deflection, stress, energy capacity. R: Juvinall ch. 7, Table 7.1.
- **Gyroscopic couple (mechanics.gyroscope)** — I: I, Ω, bearing span a, roll φ0 and T, or turning rate ω. O: M = IΩω, bearing force. R: §60-61.

**Seals (3)** — S `seals/seals_screen.dart` (modes) → E `seals/o_rings.dart`, `seal_compounds.dart`, `seal_data.dart`
- **O-ring glands (mechanical.oRings)** — I: gland type, AS568 size (349) or ID, W with tolerances, bore, groove, piston, rod, throat. O: squeeze, gland fill, stretch, gland depth L, clearance E, extrusion limit. R: Parker ORD 5700 Table 9-1, Design Charts 4-2, 4-3, 5-2-a, Fig. 3-2/3-3.
- **Quad-Ring (X-ring) glands (mechanical.quadRings)** — same outputs. F: area 0.8215·W². R: Minnesota Rubber Design Guide Sec. 6, Eqs. 1-8.
- **Rotary lip seals (mechanical.lipSeals)** — I: lip material, d, n, p. O: surface speed v, permissible speed, pressure rating. R: SKF 2021 Diagram 5 (DIN 3760).

**Standard geometries (11, lookup tables only)**
- All use one screen, S `standards/standard_table_screen.dart`. Inputs are standard and size; outputs are dimensions.
- Data files under E `standards/`: `inch_fasteners_data.dart`, `metric_fasteners_data.dart`, `washer_data.dart`, `bolt_grade_tables.dart` (with `fastening/bolt_grades.dart`), `dimension_table.dart`. Disc-spring data: E `springs/disc_spring_tables.dart`. Pipe data: E `piping/pipe_sizes.dart`, `pipe_sizes_data.dart`.
- Fasteners:
  - Hex bolts and cap screws (mechanical.hexScrews): ASME B18.2.1, ISO 4014/4017.
  - Heavy hex bolts and screws (mechanical.heavyHexScrews): B18.2.1, B18.2.6, B18.2.3.7M.
  - Socket head cap screws (mechanical.socketHeadScrews): B18.3, ISO 4762.
  - Low head socket cap screws (mechanical.lowHeadScrews): Unbrako inch, DIN 7984.
  - Button head cap screws (mechanical.buttonHeadScrews): B18.3, ISO 7380-1/-2.
  - Flat head cap screws (mechanical.flatHeadScrews): B18.3 82°, ISO 10642 90°.
  - Nuts (mechanical.hexNuts): B18.2.2, ISO 4032/4035, B18.2.4.6M.
  - Washers (mechanical.washers): B18.22.1, F436, ISO 7089/7092/7093/7094, B18.22M.
  - Bolt grades and markings (mechanical.boltGrades): SAE J429, ASTM A307/F3125/A354/A449/F1554/A193/A320/F593, ISO 898-1.
- Bellevilles: Disc spring sizes (EN 16983) (mechanical.discSpringSizes): series A/B/C, K1-K4.
- Piping: Standard pipe sizes (mechanical.pipeSizes), separate screen S `piping/pipe_sizes_screen.dart`. ASME B36.10M and B36.19. O: D, t, d, A, Af, I, J, S, Z, r, mass, water content.

### MACHINING (7; its own section since 2026-11-14, ids keep `mechanical.*`) — S `machining/` → E `machining/` (`machining.dart`, `power.dart`, `chip.dart`, `speed_tables.dart` + `speed_tables_data.dart`), E `cnc/`
- **Turning (mechanical.turning)**
  - I: tool class, speed table, material, tool, fopt/Vopt, tool-life factors, D, f, depth, motor data.
  - O: V, N = 12V/πD (inch), feed rate, Q, time, ideal Ra, Pc, Pm.
  - F: V = Vopt·Ff·Fd.
  - R: Machinery's 26th Tables 1-9, 5a-5c; power Tables 24-30.
- **Milling (mechanical.milling)** — O: V, N, vf, Q, chip thickness, engagement angle, Table 15c feed range, HSS feed, power. R: Tables 10-15e, 15a.
- **Drilling, reaming and tapping (mechanical.drilling)** — O: V, N, feed, time, thrust T, torque M, power. R: Tables 17-23, 22, 5a; Tables 31-34 (Eqs. 3-5).
- **Chip thinning and trochoidal milling (mechanical.chipThinning)** — I: D, z, W, step s, ae, ap, κr, hmax or hm, ball nose. O: fz, N, vf, engagement angle, slot advance v_net, Q. R: Machinery's pp. 1086-1088, Table 2a.
- **Speed and feed tables (mechanical.speedTables)** — lookup of Tables 1-23 and 15a.
- **G-code interpreter (machining.gcodeInterpreter)** — E `cnc/interpreter/`. Checks, explains and backplots Fanuc/Haas, LinuxCNC and Siemens programs; outputs time, extents, tools. Not a formula calculator.
- **G-code dictionary (machining.gcodeDictionary)** — E `cnc/dictionary/`. About 280 entries; reference only.

### STRUCTURAL ENGINEERING (30) — S `structural/`, `beams/`, `sections/`, `dynamics/mechanics_screens.dart` → E `structural/`, `beams/`, `sections/`, `geometry/section_builder.dart`

**Loads (ASCE 7)** — `loads_screen.dart` → `asce7_loads.dart`
- **Load combinations (structural.loadCombinations)** — I: D, L, Lr, S, R, W, E, 1.0L or 0.5L. O: LRFD and ASD governing and least. R: ASCE 7-10 2.3.2, 2.4.1.
- **Snow loads (structural.snow)** — I: risk category, terrain, exposure, thermal condition, surface, pg, θ, W, step data. O: ps, pf, pm, rain-on-snow, γ, hb, drift pd/hd/w. R: Ch. 7, Eqs. 7.3-1, 7.4-1, 7.7-1.
- **Seismic base shear (structural.seismic)** — I: risk category, site class, system (Table 12.2-1), SS, S1, TL, hn, W. O: V, Cs, SDS/SD1, seismic design category, T, R/Ω0/Cd, k. R: Chs. 11-12, Eqs. 12.8-1….

**Beams, frames, catenaries and sections**
- **Beam calculator (structural.beam)** — `beams/beam_screen.dart` → `beams/beam.dart`, `roark_check.dart`.
  - I: L, supports (pin, roller, fixed, spring k), point, distributed and couple loads, section or AISC shape, material, self-weight.
  - O: maximum M, V, deflection, σ, slope, EI, diagrams.
  - F: stiffness method with Hermite cubics.
  - R: Roark Table 8.1.
- **Frames and trusses (structural.frames)** — `frame_screen.dart` → `frame2d.dart`. I: E, ρ, nodes, members, hinges, loads. O: displacements, M, N, reactions. R: McGuire, Gallagher & Ziemian.
- **Catenaries (structural.catenary)** — S `dynamics/mechanics_screens.dart` → `dynamics/mechanics.dart` (`catenaryEnds`). I: q, l, d, sag s or H or L. O: H, V_A, V_B, T_A, T_B, sag, length, low point; parabola beside it. R: Den Hartog Mechanics §16.
- **Section builder (structural.sectionBuilder)** — `sections/section_builder_screen.dart` → `geometry/section_builder.dart`, `thin_walled.dart`. I: shape dimensions with fillets. O: A, Ix, Iy, Sx, Sy, Zx, Zy, r, J, Cw, centroid, shear centre xo/yo, r̄o, Ixy, principal axes. R: Roark A.1/10.1; AISC Design Guide 9.

**Tensile elements** — `tensile_screen.dart` → `tensile.dart`; cables C `profiles/cable_catalog_data.dart`
- **Frequency and tension (structural.tensileFrequency)**
  - I: construction (rod, bar, cable), size, material, d, A, EI, L, measured f, mode n, end condition.
  - O: T, taut-string T, μ = ρA, T/A, ξ = L√(T/EI).
  - F (pinned): f = n²π/(2L²)·√(EI/μ)·√(1 + TL²/(n²π²EI)); string limit f = n/(2L)·√(T/μ). Clamped ends are solved by bisection on the frequency equation.
  - R: Den Hartog Vib. §31.
- **Elongation and stiffness (structural.tensileElongation)** — O: δ = TL/(EA) + αΔT·L, k = EA/L, T/A, strain, breaking load, μ.

**Connections and plates** — `connections_screen.dart` → `connections.dart`, `groove_welds.dart`, `prequalified_joints.dart`, `aisc360.dart`
- **Bolts (AISC 360) (structural.bolts)** — I: bolt group (Table J3.2), d, N/X threads, bearing or slip-critical, Class A/B, holes, ns, V, T. O: shear, tension, combined, slip. R: AISC 360-22 J3, Eqs. J3-1 to J3-5.
- **Bolted plate (structural.boltedPlate)** — O: bearing/tearout, yield, net rupture, block shear. R: J3.10, J4.1-J4.3.
- **Fillet welds (AISC 360) (structural.filletWelds)** — I: end-loaded, w, l, FEXX, θ, tmin, P. O: strength, throat, kds, long-weld β, sizes. R: J2.2, J2.4, Tables J2.4/J2.5.
- **Groove welds (AISC J2) (structural.grooveWelds)** — CJP, PJP, flare. O: strength, throat, matching filler, prequalified joint. R: J2.1-J2.6; AWS D1.1:2010 Figs. 3.3/3.4, Table 3.1.
- **Column base plates (structural.basePlates)** — I: Pu, B, N, d, bf, f′c, A2, Fy. O: minimum tp, bearing, m, n, λn′. R: AISC J8; Manual Part 14; DG1.
- **Plate buckling (structural.plateBuckling)** — I: case, a, b, t, E, ν, Fy. O: σ′, τ′, K, critical load. R: Roark Table 15.2.

**Column calculator** — `column_screen.dart` → `columns.dart`, `aisc360.dart`
- **Column buckling (structural.columnBuckling)** — I: AISC shape or section, E, Sy, L, K or GA/GB. O: Pcr, σcr, K, KL/r, (KL/r)1, Euler, Johnson, factor of safety, plus AISC Ch. E. R: Shigley §4-11 to 4-15.
- **Eccentric column loading (structural.columnEccentric)** — I adds P, e. O: σmax by the secant formula, load to first yield, ec/k², short strut. R: Shigley Eqs. 4-49 to 4-56.
- **Effective length and slenderness (structural.columnSlenderness)** — O: K, KL/r, (KL/r)1, KL. R: AISC Commentary Table C-A-7.1, Eqs. C-A-7-1 to C-A-7-3.

**Member design** — `member_design_screen.dart` → `aisc360.dart`, `aisc370.dart`, `adm2015.dart`, `lateral_torsional_buckling.dart`
- **Steel member design (AISC 360) (structural.steelMember)** — I: AISC shape, Fy, Fu, Lcx, Lcy, Lcz, Lb, Cb, loads, LRFD/ASD. O: tension, compression, flexure about x and y, shear, H1.1 interaction. R: AISC 360-22 D, E3/E4/E7, F2-F11, G, H1.1, Table B4.1.
- **Stainless steel member design (AISC 370) (structural.stainlessMember)** — R: AISC 370-21 (Table E3.1, F2.1).
- **Aluminum member design (ADM) (structural.aluminumMember)** — I: ADM shape, alloy (Table A.3.3), Fty, Ftu. R: ADM 2015 Part I.
- **Lateral-torsional buckling (structural.lateralBuckling)** — I: AISC or EN shape, Fy, E, G, Lb, Cb, load height. O: φbMn, Mn/Ωb, Mn, Mcr, Lp, Lr, Cb. R: AISC F1-F11; ADM F.4; SSRC Guide Ch. 5.

**Standard shapes (lookup)** — `sections/aisc_shapes_screen.dart`, `shape_table_screen.dart`
- AISC steel shapes (structural.aiscShapes): `aisc_shapes_data.dart`, AISC v16.0, 2,299 shapes.
- EN steel shapes (structural.enShapes): `en_shapes_data.dart`, ArcelorMittal / EN 10365.
- AISI cold-formed steel (structural.coldFormedShapes): `sfia_shapes_data.dart`, SFIA v2025.1, AISI S100-16.
- ADM aluminum shapes (structural.admShapes): `adm_shapes_data.dart`, ADM 2015 Part V, 740 shapes.

**Wind (ASCE 7-10)** — `wind_screen.dart` → `asce7_wind.dart`
- **Wind velocity pressure (structural.windVelocity)** — I: V, z, exposure, topography (H, Lh, x), Kd. O: qz = 0.00256·Kz·Kzt·Kd·V² (Eq. 27.3-1), Kz, Kzt, Kd.
- **Wind on buildings (MWFRS) (structural.windBuilding)** — O: qh, G, ±GCpi, wall and roof pressures. R: Eq. 27.4-1, Fig. 27.4-1, Eq. 26.9-6.
- **Wind on components and cladding (structural.windComponents)** — I: effective area A. O: zone pressures, end-zone a. R: Ch. 30 Pt 1, Figs. 30.4-1, 30.4-2A.
- **Wind on other structures (structural.windOther)** — signs, chimneys, tanks, lattice, towers. O: F, Cf, G. R: Ch. 29, Eqs. 29.4-1, 29.5-1.

### THERMAL ENGINEERING (18; not in the outline) — S `thermal/` → E `thermal/`
**Properties**
- **Steam tables (thermal.steamTables)** — → `if97.dart`, `if97_data.dart`.
  - I: any pair of p-T, p-h, p-s, h-s, T-x, p-x; table range and step.
  - O: T, p, phase, x, v, ρ, h, u, s, cp, cv, w.
  - R: IAPWS-IF97 (2007), regions 1-5.
- **Refrigerants (thermal.refrigerants)** — → `refrigerants.dart`, `refrigerant_data.dart`. 22 fluids by Helmholtz equations of state (CoolProp coefficients). Same outputs as steam.
- **Ideal gases (thermal.idealGases)** — → `ideal_gases.dart`, `ideal_gas_data.dart`. 19 gases. O: cp, cv, k, ρ, R, h, u, s°, s(T,p), speed of sound. R: NASA TM-4513 polynomials.

**Thermodynamics** — `thermo_screens.dart`
- **Steady-flow devices (thermal.steadyFlow)** — → `flow_devices.dart`. Turbine, compressor, pump, nozzle, diffuser, valve, heater, mixer. I: p1, T1, p2, T2, η, V1, V2, ṁ. O: outlet state, w, q, exit velocity, x, sgen, exergy destroyed, power. R: Marks pp. 4-5, 4-24.
- **Gas processes and mixing (thermal.gasProcesses)** — → `gas_processes.dart`. Isentropic, isothermal, constant p, constant v, polytropic n, with cp(T). O: T2, p2, boundary and shaft work, Q, Δu, Δh, Δs, mixture properties, mixed temperature, sgen. R: Marks 4.1.

**Power cycles**
- **Rankine cycle (thermal.rankine)** — → `rankine.dart`. I: p1, T1, pc, ηt, ηp, reheat, feedwater heaters. O: η, net w, steam rate, heat rate, turbine and pump work, qin, qout. F: η = (hc − hd)/(hc − ha). R: Marks p. 4-19.
- **Gas cycles (thermal.gasCycles)** — → `gas_cycles.dart`. Otto, Diesel, dual, Brayton (regeneration, intercooling, reheat), Stirling, Ericsson, Carnot. I: cp, k, T1, p1, T3, r, rc, rp, η. O: η, net w, qin, qout, MEP, back work ratio, peak p and T. R: Marks 4.1 pp. 4-10 to 4-13.
- **Combined cycle (thermal.combinedCycle)** — → `combined_cycle.dart`. I: T1, p, rp, T3, ηc, ηt, gas cp/k, steam conditions, pinch. O: combined, gas and steam η, ms/mg, stack temperature.

**Refrigeration** — `refrigerant_screens.dart` → `refrigeration.dart`
- **Vapour-compression cycle (thermal.vaporCompression)** — I: refrigerant, Te, Tc, ΔTsh, ΔTsc, ηc, Q. O: COP cooling and heating, qL, w, qH, pressures, discharge temperature, flash x, ṁ. R: Marks p. 4-18.
- **Gas refrigeration (thermal.gasRefrigeration)** — reversed Brayton. I: gas, T1, p1, rp, T3, ηc, ηt, regenerator ε. O: COP, qL, w, coldest gas, ṁ, power.

**Heat transfer** — R for all: Lienhard & Lienhard, A Heat Transfer Textbook, 6th ed.
- **Thermal resistance (thermal.resistance)** — `thermal_resistance_screen.dart` → `resistance.dart`. I: Ti, To or Q, hi, ho, A or radii, layers (k from library). O: Q, R total, R-value, UA, U, interface temperatures. R: Eqs. 2.16, 2.22, 2.26.
- **Conduction (thermal.conduction)** — `conduction_screens.dart` → `conduction.dart`. Layered walls, cylinders and spheres with contact R″; critical radius; heat generation.
- **Convection (thermal.convection)** — `convection_screens.dart` → `convection.dart`, `transport.dart` (+ `transport_data.dart`).
  - I: configuration, fluid, T∞, Tw, L or D.
  - O: h, Nu, Re, Ra, Pr, flux, fluid properties.
  - Correlations: Churchill-Chu, Churchill-Bernstein, Gnielinski, Raithby-Hollands, flat plate, Whitaker.
- **Fins (thermal.fins)** — → `fins.dart`. Straight, pin, triangular, annular (Bessel functions). O: Q, ηf, εf, tip temperature, resistance. R: Eqs. 4.44-4.61.
- **Radiation (thermal.radiation)** — → `radiation.dart`. O: σT⁴, Wien peak, band fraction, view factors (Tables 10.2/10.3), gray pairs, shields, 3-surface enclosure.
- **Heat exchangers (thermal.heatExchangers)** — `convection_screens.dart` → `heat_exchangers.dart`. O: Q, area, outlets, ε, NTU = UA/Cmin, Cr, LMTD, F. R: Ch. 3 Table 3.1.
- **Transient heating (thermal.transient)** — `conduction_screens.dart` → `transient.dart`. Lumped, exact series for slab/cylinder/sphere, semi-infinite. O: T, τ, Bi, Θ, Q, Fo, time to reach a temperature.

**Thermal stress**
- **Thermal expansion and stress (thermal.thermalStress)** — → `thermal_stress.dart`. I: case, E, ν, α, ΔT, materials A/B. O: αΔT, restrained and gradient stresses, fits, bimetallic strip. R: Roark 7th §16.6 cases 1-5, 9, 15, 16, 19, 20; §8.2.

### ELECTRICAL ENGINEERING (50) — S `electrical/` → E `electrical/`. Values are SI with prefixes (no unit system).

**Voltage, resistance and current (11)**
- **Ohm's law (electrical.ohmsLaw)** — `ohms_law_screen.dart` → `circuits.dart`. I: any two of V, I, R, P. O: the other two. F: V = IR, P = VI. R: Art of Electronics Eqs. 1.1-1.2.
- **Series and parallel (electrical.seriesParallel)** — R, C, L in series and parallel; n equal; value needed for a total with nearest E24/E96. R: AoE Eqs. 1.3-1.5, 1.17-1.18.
- **Voltage and current dividers (electrical.dividers)** — I: Vin, R1, R2, RL, Vout, I. O: Vout loaded and unloaded, Rth, powers, R1 for a wanted Vout, branch currents. R: AoE Eq. 1.6.
- **Wheatstone bridge (electrical.wheatstone)** — I: Vs, R1-R3, Rx, Rg. O: Rx at balance, Vout, Rth, Ig. R: Kuphaldt I §8.10.
- **Δ-Y transform (electrical.deltaWye)** — O: RA, RB, RC or RAB, RBC, RCA. R: Kuphaldt §10.13.
- **Wire resistance and voltage drop (electrical.wire)** — `wire_screen.dart` → `wire.dart`.
  - I: material ρ20 and α, AWG (B258 gauge law) or mm² (IEC 60228) or d, l, T, I, V, t.
  - O: R conductor = ρ·l/A, R loop, ΔV = I·R, P loss = I²R, Q = I²Rt, ΔV/V, V load.
  - R: Kuphaldt §12.5-12.6, Tables 12.4/12.5.
- **Resistor of any section (electrical.resistor)** — `resistor_screen.dart` → `conductor_heating.dart`. O: R(T) = ρ(T)l/A, P, self-heated temperature, ρ from a measured R. R: CDA Pub. 22.
- **Cable ampacity tables (electrical.cableAmpacity)** — `cable_ampacity_screen.dart` → `ampacity_tables.dart`, `nec_ampacity_data.dart`. I: table, rating, size, Ta, Ib, soil resistivity, grouping. O: ampacity, smallest size. F: ambient correction √((Tc − Ta)/(Tc − Tref)). R: NEC 2011 Tables 310.15(B)(16)-(21); IEC 60364-5-52 Annex B.
- **Ampacity of a conductor (electrical.ampacity)** — `ampacity_screen.dart` → `conductor_heating.dart`.
  - I: ρ20, α, section (w, t, d, di, A, perimeter P, h), T, Ta, air speed, emissivity.
  - O: I, T, θ, J, P′, R′, skin depth.
  - F: Wv = 7.66θ^1.25/L^0.25; Wh = 5.92θ^1.25/L^0.25; forced Wa = 120√v·A·θ; radiation 5.70·10⁻⁸·e(T⁴ − Ta⁴); I = √(Q/(S·ρ(T)/A)).
  - R: CDA Pub. 22 Sec. 2.2.
- **PCB trace heater (electrical.pcbHeater)** — `pcb_heater_screen.dart` → `pcb_heater.dart`. I: Tpeak, Ta, shape, size, tr, P, V, board, minimum w and gap, pattern. O: board size, trace, R, I, steady temperature, rise; DXF/CSV export. F: RK4 lumped thermal model. R: CDA Pub. 22.
- **Resistor codes and values (electrical.resistorCodes)** — → `resistor_codes.dart`. Colour bands both ways, 3/4-digit and EIA-96 markings, nearest E3-E192 value. R: TT Electronics TN004, IEC 60062.

**Power (6)** — → `power.dart`
- **Energy and Joule heating (electrical.jouleHeating)** — Q = I²Rt in J and Wh.
- **AC power (electrical.acPower)** — I: V, I or P, pf, single- or three-phase, wye/delta. O: P, Q, S, φ, phase values. R: AoE Eq. 1.34; Kuphaldt.
- **Power factor correction (electrical.pfCorrection)** — I: P, pf1, pf2, V, f. O: Qc, C (per phase), S and I before and after. R: Kuphaldt II §11.4.
- **RMS and average values (electrical.waveforms)** — sine, square, triangle, sawtooth, rectified, pulse with duty D. O: RMS, average, peak, peak-to-peak, crest factor, form factor.
- **Decibels (electrical.decibels)** — dB ratios and back; dBm, dBW, dBV, dBu, dBµV into 50/75/600 Ω. R: AoE Eqs. 1.11-1.12.
- **Battery life (electrical.battery)** — I: C, I, usable share u, V. O: t, Q, E, E usable, P. R: Kuphaldt §11.3.

**Insulation (2)** — → `insulation.dart`
- **Creepage and clearance (electrical.creepage)** — I: U, Uimp, overvoltage category, pollution degree, material group (CTI), basic/reinforced, altitude. O: clearance, creepage, impulse voltage, solid-insulation test voltages. R: IEC 60664-1:2007 Tables F.1/F.2/F.4/A.2; IEC 61010-1:2010 Tables 3-7, K.2-K.17.
- **Insulation thickness (electrical.insulationThickness)** — F: V = Vnom·(t/tnom)^n, n = 0.5 by default. I: insulator, Vnom, tnom, t, V, n. R: Zhao & Liu 2020.

**Magnets and inductors (13)** — `inductance_screens.dart`, `field_screens.dart`, `magnetic_circuit_screen.dart`, `induction_screen.dart` → `inductance.dart`, `magnetics.dart`, `bitter.dart`, `magnetic_circuits.dart`, `induction.dart`

*Inductance* (R: Rosa & Grover 1912, formula numbers cited)
- **Inductance of wires and lines (electrical.wireInductance)** — straight wire (94)-(97), two-wire line (100), coax (119), parallel-wire mutual (98).
- **Inductance of loops (electrical.loopInductance)** — circle (61), rectangle (107)/(108).
- **Inductance of coils (electrical.coilInductance)** — Lorenz solenoid (73) with Nagaoka K, Brooks & Turner multilayer, toroid (83), Bitter stack. O: L, wire length, R, τ.
- **Mutual inductance (electrical.mutualInductance)** — coaxial loops, Maxwell (1), elliptic integrals.
- **Planar spiral inductance (electrical.spiralInductance)** — I: d_out, d_in, n. O: L (Mohan et al. 1999), Wheeler L, d_avg.
- **Inductor energy and EMF (electrical.inductorEnergy)** — E = ½LI², V = L·di/dt, τ = L/R. R: OpenStax UP2 §14.2-14.4.
- **Motional EMF and induction (electrical.induction)** — rod on rails, generator, loops, flux-change charge, ideal transformer. R: Purcell & Morin Secs. 7.2-7.3.

*Magnet particulars*
- **Permanent magnet field (electrical.permanentMagnet)** — cylinder, ring, block on the axis. I: MMPA 0100-00 grade (Table IV-1) or Br, D, a, d, L, z. O: B(z), B at the face, Fmax = B²A/2μ0.
- **Magnetic pressure and hoop stress (electrical.coilForces)** — O: p = B²/2μ0, hoop T, σ hoop, ΔR = pR²/(Et), Bitter plate stresses. R: OpenStax §14.3; Roark 13.1 case 1b, 16.2 case 8.
- **Magnetic circuits (electrical.magneticCircuits)** — gapped and shell cores. O: N·I, Φ, reluctance, L, pull, gap energy. R: Purcell & Morin 11.10-11.11 (iron B-H curve).

*Biot-Savart* (R: OpenStax UP2 §12.2-12.4)
- **Field of a straight conductor (electrical.conductorField)** — B of a wire or segment, F/l between parallel wires.
- **Field of a loop (electrical.loopField)** — B on the axis, Helmholtz pair.
- **Field of a solenoid (electrical.solenoidField)** — thin and thick windings anywhere on the axis, Bitter.

**Standard packages (10, lookup)**
- S `standards/standard_table_screen.dart`, data E `standards/electronic_packages_data.dart`.
- Chip resistors and capacitors (electrical.chipPackages), SOD and SOT (electrical.sotPackages), SOIC/SSOP/TSSOP/MSOP (electrical.soPackages), QFP (electrical.qfpPackages), QFN/DFN (electrical.qfnPackages), DPAK/D2PAK/SOT-223 (electrical.powerPackages), DIP (electrical.dipPackages), TO (electrical.toPackages), axial (electrical.axialPackages), radial capacitors (electrical.radialPackages). JEDEC MS/MO outlines.

**Practical electronics (8)** — `practical_screens.dart` → `practical.dart`; `rlc_screen.dart` → `rlc.dart`
- **LED and Zener resistors (electrical.ledZener)** — R = (Vs − n·Vf)/I (next E24 up), P; Rs = (Vin min − Vz)/(IL max + Iz min), Pz max. R: Kuphaldt III §3.11-3.12.
- **LM317 regulator (electrical.lm317)** — Vout = Vref·(1 + R2/R1) + I_ADJ·R2; headroom; P = (Vin − Vout)·I. R: TI SLVS044Z Eq. 1.
- **555 timer (electrical.timer555)** — f ≈ 1.44/((RA + 2RB)C); t high = 0.693(RA + RB)C; duty cycle; monostable t = 1.1RC. R: TI SLFS022K.
- **Op-amp gain (electrical.opAmp)** — G = −Rf/Rin, 1 + Rf/Rg, or 1; G in dB; bandwidth = GBW/(1 + Rf/Rin). R: AoE Eqs. 4.1-4.2.
- **RC, RL and LC circuits (electrical.reactive)** — τ = RC or L/R; f−3dB; V(t); f0 = 1/(2π√LC); Z0 = √(L/C); Q; X_C, X_L. R: AoE Eqs. 1.20-1.37.
- **RLC circuits (AC) (electrical.rlcCircuits)** — series, parallel, coil with C. O: |Z|, I, φ, P, pf, resonance, Q, bandwidth f1/f2, damped step response. R: Purcell & Morin Sec. 8.1.
- **ADC resolution (electrical.adc)** — LSB = FS/2^N; noise q/√12; SNR = 6.02N + 1.76; ENOB = (SINAD − 1.76)/6.02. R: Kuphaldt IV §13.10.
- **PCB trace width (electrical.traceWidth)** — I = k·ΔT^0.44·A^0.725 (k = 0.048 external, 0.024 internal; A in mil²); w = A/t; R, ΔV, P. R: IPC-2221 curve fit.

### SCIENCE (0)
Not implemented: there is no Science section, group or calculator id anywhere in the catalog or the code. The nearest built content is Materials (periodic table, diffusion, corrosion, materials science from Ashby), Physics and Thermal engineering.

### MATHEMATICS (23) — S `math/` → E `math/` (`expression.dart`, `numbers.dart`, `polynomial.dart`, `calculus.dart`, `trig_geometry.dart`, `matrix.dart`, `ode.dart`, `grapher.dart`, `curve_fit.dart`, `special.dart`, `pde/`). R: Marks 10th Sec. 2 unless noted.

*Calculators and plotting*
- **RPN and algebraic calculator (math.rpl)** — `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/lib/calculator/rpl_screen.dart` + `python/lacewing/rpl.py`. HP 48/50-style RPL with units (pint) and symbolic algebra (SymPy).
- **Equation grapher (math.grapher)** — up to 8 curves; roots, extrema, integrals, intersections, area between, tangents.
- **Data plot and curve fit (math.dataPlot)** — CSV/xlsx import; linear, polynomial 2-6, exponential, power, logarithmic, reciprocal; plane and quadratic surface. O: coefficients, R, R², adjusted R², RMS error. F: Householder QR.

*Foundational concepts*
- **Expression calculator (math.calculator)** — decimal, fraction, binary, hex.
- **Numbers and fractions (math.numbers)** — prime factors, GCD, LCM, fraction, significant figures, bases.
- **Permutations and combinations (math.combinatorics)** — n!, nPr, nCr, with repetition, binomial distribution P(X = k), mean, variance.

*Algebra*
- **Polynomial roots (math.polynomial)** — any degree (Aberth-Ehrlich).
- **Solve an equation (math.solve)** — f(x) = g(x), all roots in an interval (Brent).
- **Linear interpolation (math.interpolate)** — y at x, x at y, slope.

*Geometry*
- **Coordinate geometry (math.coordinates)** — distance, midpoint, slope, line, point-line foot, circle through 3 points, line intersection.
- **Circular segments (math.segment)** — r, θ, c, h → A, s, centroid, fraction of the circle (p. 2-8).

*Trigonometry*
- **Triangle solver (math.triangle)** — SSS, SAS, ASA, AAS, SSA (two triangles) → sides, angles, area, r, R, heights, medians.
- **Trigonometric functions (math.trig)** — six functions, inverses, reference angle.

*Calculus*
- **Derivatives and integrals (math.calculus)** — f′, f″ (Richardson), ∫ (Gauss-Kronrod and Simpson), arc length, roots, extrema.
- **Series and sums (math.series)** — Σ finite or infinite, convergence (Levin).

*Linear algebra*
- **Matrix calculator (math.matrix)** — up to 6×6: det, rank, trace, inverse, eigenvalues and vectors, Ax = b, A·B.
- **Vectors (math.vectors)** — a·b, a×b, angle, projection, triple product.

*Differential equations*
- **Numerical ODE solver (math.ode)** — y′ or y″, Dormand-Prince 5(4).
- **Second-order linear equations (math.linearOde)** — y″ + 2b·y′ + a²·y = c·sin kx in closed form (p. 2-32 types 17-19).

*Partial differential equations* (R: Farlow, PDEs for Scientists and Engineers)
- **Heat equation (1-D) (math.heatEquation)** — eigenfunction series and Crank-Nicolson.
- **Wave equation (1-D) (math.waveEquation)** — D'Alembert and standing waves.
- **Laplace and Poisson equations (math.laplaceEquation)** — rectangle, disk, annulus; series and relaxation.
- **Fourier series and transforms (math.fourierSeries)** — coefficients, Gibbs, Parseval, transforms.

### PHYSICS (40) — S `physics/` (all ModeCalculatorScreen) → E `physics/` (`constants.dart` CODATA 2018 + NASA bodies, `motion.dart`, `waves.dart`, `thermal.dart`, `fields.dart`, `optics.dart`, `modern.dart`, `kinetic_theory.dart`, `entropy.dart`, `phase_change.dart`, `expansion_fits.dart`, `engine_cycles.dart`, `potential.dart`, `currents.dart`, `magnetic_forces.dart`, `motors_lc.dart`, `ampere.dart`, `electrostatics.dart`, `capacitance.dart`, `image_charges.dart`, `dielectrics.dart`, `em_waves.dart`, `quantum.dart`, `atoms.dart`, `nuclear.dart`)
- Sources: OpenStax University Physics Vols. 1-3 for the first 20, Serway & Jewett 7th for 15, Purcell & Morin 3rd for 5.

*Motion*
- **Constant acceleration (physics.kinematics)** — u, a, t, v, s, h, g. O: v, s, v avg, a, t, h max.
- **Projectile motion (physics.projectile)** — v0, θ0, h, g. O: t, R, H, impact v and angle θv.
- **Circular motion (physics.circular)** — r, v, m, θ, g. O: ac, Fc, ω, T, n, μs, ideal bank θ.

*Gravitation*
- **Gravitation and orbits (physics.gravitation)** — g at altitude, v esc, v orbit, T, E, orbit radius for a period, F.

*Oscillations and waves*
- **Pendulums and springs (physics.oscillators)** — T0, exact large-swing T (AGM), g from T, physical pendulum, mass-spring.
- **Strings and air columns (physics.standingWaves)** — v = √(FT/μ), f1, λ1, open/closed tubes with air temperature.
- **Sound level and Doppler (physics.sound)** — β from I or Δp, point source, Doppler fo.

*Thermal physics*
- **Heat and expansion (physics.heat)** — Q = mcΔT + mL, mixing Tf, ΔL, ΔV.
- **Conduction and radiation (physics.heatTransfer)** — P = kAΔT/d, R, net radiation.
- **Ideal gas (physics.idealGas)** — pV = nRT, v rms, isothermal, adiabatic and isobaric W, Q, ΔU.
- **Heat engines (physics.heatEngines)** — Carnot e, Qh, Qc, KP, KR.
- **Kinetic theory of gases (physics.kineticTheory)** — Maxwell-Boltzmann speeds, distribution share (erf), CV/CP, γ, mean free path, Boltzmann factor, law of atmospheres.
- **Entropy (physics.entropy)** — ΔS of heating, phase change, gas state change, free expansion, reservoirs, mixing.
- **Calorimetry with phase change (physics.phaseChangeCalorimetry)** — ice, water, steam and a container: Tf or the phase amounts left, ΔS.
- **Expansion: gaps, fits and containers (physics.expansionFits)** — ΔT to close a gap or free a fit, liquid overflow, sealed can p2, gas thermometer.
- **Real engines and refrigerators (physics.realEngines)** — e against Carnot, COP, PV-path work.

*Electricity and magnetism*
- **Charges and fields (physics.charges)** — Coulomb F, E, V, U, plate acceleration K and v.
- **Potential of point charges (physics.pointChargePotential)** — up to 6 charges: V, E, U, W.
- **Parallel-plate capacitor (physics.capacitor)** — C = κε0A/d, Q, E, U.
- **Fields of charge distributions (physics.chargeDistributions)** — rod, line, ring, disk, sphere, shell, cylinder, sheet, slab, dipole, quadrupole.
- **Capacitance of conductor shapes (physics.capacitance)** — plates with edge fields, spheres, spheroid, coax, two wires, wire over a plane, layered dielectrics, breakdown.
- **Image charges (physics.imageCharges)** — plane, corner, two planes, sphere: q′, σ, F, W.
- **Dielectrics and polarization (physics.dielectrics)** — χe, P, σb, D, sphere in a field, slab force, Clausius-Mossotti.
- **Charged particles in a magnetic field (physics.particleInField)** — F, r, T, f, cyclotron, induced ε.
- **Current and resistance (physics.currentResistance)** — vd, n, J, ρ(T), coax leakage, heaters.
- **Magnetic forces and torques (physics.magneticForces)** — F on a wire, coil torque, dipole U, Hall ΔVH, velocity selector, mass spectrometer.
- **Ampère's law (physics.ampereLaw)** — round conductors, tubes, coax, toroids.
- **Motors, LC circuits and field energy (physics.motorsLc)** — back emf, LC ω0, damping, uE and uB.
- **Electromagnetic waves (physics.emWaves)** — S, E, B, H, u, v, n, η, radiation pressure, Fresnel R/T, VSWR, skin depth δ.

*Light and optics*
- **Refraction (physics.refraction)** — Snell θ2, θc, Brewster θB, v2.
- **Thin lenses and mirrors (physics.lenses)** — di, m, hi, P, lens-maker R.
- **Interference and diffraction (physics.interference)** — double and single slit, grating, Rayleigh θR.

*Modern physics*
- **Special relativity (physics.relativity)** — β, γ, Δt, L, p, E0, K.
- **Photons and matter waves (physics.photons)** — E, f, p, photoelectric K max and Vs, Compton Δλ, de Broglie λ.
- **Radioactivity and binding energy (physics.nuclear)** — λ, τ, A, N(t), BE, BE per nucleon.
- **Blackbody radiation (physics.blackbody)** — Wien λmax, Stefan M and P, Planck I(λ), band fraction.
- **Quantum wells and tunnelling (physics.quantumWells)** — particle in a box En, oscillator, T ≈ e^(−2CL), uncertainty.
- **Hydrogen atom and X-rays (physics.hydrogen)** — Bohr En and rn, transitions and series, quantum numbers, Moseley X-rays.
- **Molecules and solids (physics.molecules)** — rotational and vibrational levels, Fermi EF/vF/TF, band gap λ.
- **Nuclear structure and reactions (physics.nuclearReactions)** — radius, density, liquid-drop Eb, Q values, fission and fusion energy, dose D and H.

### UNIT CONVERTER (1)
- **Unit converter (units.converter)** — S `units/unit_converter_screen.dart` → E `units/unit_definitions.dart` (Pint-format definitions, each line tagged with its source), `unit_registry.dart` (parser, prefixes, plurals, temperature offsets, mixed "5 ft 2 in", typed expressions such as "3 ft + 2 in to mm"), `unit_categories.dart`, `dimension.dart`.
- I: category, value, from, to, significant figures. O: the value in every unit and in SI base units.
- R: BIPM SI Brochure 9th ed. (2019), NIST SP 811 (2008) App. B.8, CODATA 2018, IEC 80000-13, IAU 2012 B2.
- **48 categories in 7 groups:**
  - **Space and time (8):** Length, Area, Volume, Angle, Time, Speed, Acceleration, Frequency and rotation.
  - **Mechanics (12):** Mass, Force, Pressure and stress, Torque and moment, Energy and work, Power, Density, Mass per length, Stiffness and line load, Second moment of area, Section modulus, Mass moment of inertia.
  - **Fluids (5):** Volumetric flow, Mass flow, Dynamic viscosity, Kinematic viscosity, Fuel economy (converted through 1/x).
  - **Thermal (6):** Temperature, Temperature difference, Thermal conductivity, Heat transfer coefficient, Specific heat, Thermal expansion.
  - **Electrical and magnetic (10):** Electric current, Voltage, Resistance, Electrical resistivity, Charge, Capacitance, Inductance, Magnetic flux, Magnetic flux density, Magnetic field strength.
  - **Light and radiation (4):** Illuminance, Luminance, Radioactivity, Absorbed and equivalent dose.
  - **Other (3):** Data, Amount of substance, Ratio.

### Outside the catalog but relevant
- **"My calculators"** (user macros, `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/lib/macros/`): Python worksheet macros with typed inputs and steps (`symbol = expression`). Example and template: `/Users/admin/Developer/Tension_Meter/App/apps/lacewing_lab/assets/macros/examples.py`, `template.py`. This is a ready template format.
- **Lookup tables** sit in the data files named under each entry, generated by the scripts in `/Users/admin/Developer/Tension_Meter/App/scripts/`: AISC/EN/SFIA/ADM shapes, threads, fasteners, washers, fits, machining speeds, pipe, seals, IF97, refrigerants, ideal gases, transport properties, NEC/IEC ampacity, elements, gear J factors, crack factors, disc springs.
- Source documents are in `/Users/admin/Developer/Tension_Meter/App/Documents and Background/Material Supplemental Data/library/` and `web/`.

---

## 3. Outline compared with the implementation
- **Science, Mathematics, Physics and Unit Convertor** are empty in the outline. Mathematics has 23 calculators, Physics 40 and the Unit converter 1 (48 categories). Science has nothing built.
- **Geometry:** matches. 2D Properties of Areas has the outline's four. 3D adds "Composite solids" beside "Properties of solids".
- **Mechanical engineering:**
  - Von Mises and Fatigue are built, plus 6 more under Stress and failure.
  - "Threads" moved under Fastening as Thread strength, any size, and Power screws.
  - "Bolt Torque and Clamping forces" became Bolt torque (simple), Bolt torque and clamping, VDI 2230, Bolt groups and Riveted joints.
  - "Welds" became 4 calculators.
  - "Belleville Springs" moved to the Springs group (id `mechanical.discSprings` unchanged).
  - "Machining" became its own top-level section (7).
  - Hydrodynamics "Depth, Pressure", "Buoyancy" and "Fluid Drag" are built, plus Pipe flow.
  - "Statics" has no calculator; it was removed on purpose (CLAUDE.md line 219). The Mechanics group (capstan, impact, gyroscope, impact loading) is the nearest.
  - "Dynamics" became Dynamics > Vibrations (6).
  - Not in the outline: Strain measurement, Springs, Pressure vessels, Bearings, Shafts, Fits and tolerances, GD&T, Clutches and brakes, Belts and chains, Gear design, Seals, Standard geometries.
- **Structural engineering:**
  - Beam calculator: built.
  - Column calculator: Eccentric loading, Buckling and Slenderness ratio are built.
  - Tensile elements: Frequency and tension, Elongation and stiffness are built.
  - Catenaries: built.
  - "Wind: Velocity, pressure, and drag" became 4 ASCE 7-10 calculators. Drag lives in Hydrodynamics > Fluid drag.
  - Not in the outline: Loads (3), Frames and trusses, Section builder, Connections and plates (6), Member design (4), Standard shapes (4).
- **Electrical engineering:**
  - Voltage, Resistance and Current (11), Power (6) and Practical Electronics (8) are built.
  - Magnets and Inductors: Inductance (7), Magnet particulars (3), Biot-Savart (3) are built.
  - Not in the outline: Insulation (2), Standard packages (10).
- **Not in the outline at all:** Materials (10) and Thermal engineering (18).
- **Planned but not built:** nothing in the catalog. The code and docs do list some sub-features as not built:
  - VDI 2230 Part 2 (multi-bolt).
  - Constant-force and volute springs.
  - Buckling under external pressure.
  - Beam internal hinges, varying EI and shear deformation.
  - Lathe tool-nose compensation in the G-code interpreter.

## 4. Count of calculators per heading (total 286)

| Heading | Calculators |
|---|---|
| Materials | 10 (library/data 3, materials engineering 7) |
| Geometry | 6 (2D 4, 3D 2) |
| Mechanical engineering | 101 |
| Machining | 7 |
| Structural engineering | 30 |
| Thermal engineering | 18 |
| Electrical engineering | 50 |
| Science | 0 |
| Mathematics | 23 |
| Physics | 40 |
| Unit converter | 1 (48 categories) |

- **Mechanical engineering (101) by group:** Stress and failure 8, Strain 2, Fastening 12, Springs 10, Pressure vessels 4, Bearings 7, Shafts 5, Fits 2, GD&T 6, Clutches and brakes 6, Belts and chains 5, Gears 6, Hydrodynamics 4, Vibrations 6, Mechanics 4, Seals 3, Standard geometries 11.
- **Structural engineering (30) by group:** Loads 3, Beams/frames/catenaries/sections 4, Tensile 2, Connections 6, Columns 3, Member design 4, Standard shapes 4, Wind 4.
- **Electrical engineering (50) by group:** Voltage, resistance and current 11, Power 6, Insulation 2, Magnets and inductors 13, Standard packages 10, Practical 8.
- **Lookup-only or reference pages, which a converter may treat as data rather than templates (37):** 11 standard geometries, 4 standard shapes, 10 packages, Speed and feed tables, G-code dictionary, G-code interpreter, GD&T symbol reference, Material library, Periodic table, and the free-form math tools (RPN calculator, Expression calculator, Equation grapher, Data plot, Equation solver, ODE solver).

## Limits of this inventory
- Inputs and outputs above are names and symbols from the help files. Several lists were truncated where they are very long (GD&T frame, VDI 2230, rolling bearings, thread strength, fatigue); the full lists are in each `H/<id>.md`.
- **Units:** the engines work in SI base units and the page shows the unit by `Dim` (`S/dims.dart`). Help files do not state units. Electrical and physics values are SI with prefixes. Mechanical pages switch SI/Imperial.
- The formulas shown are the compact ones from help and code. The complete per-calculator formula strings are in the `Formula(...)` and `WorkingLine(...)` calls of the files named, and the grep command in section 1 pulls them.

## 5. Conversion status (ADR-0014 item 7)

A calculator has "Templates" once at least one built-in template names it in `from`; what each template leaves out is in its own prose. Each converted template gets the owner's review.

<!-- status:begin (scripts/lacewing-status.py writes this table) -->

132 of 286 calculators have at least one template (373 templates).

| Section | Calculators | With templates |
|---|---|---|
| Materials | 10 | 0 |
| Geometry | 6 | 1 |
| Mechanical engineering | 101 | 100 |
| Machining | 7 | 0 |
| Structural engineering | 30 | 30 |
| Thermal engineering | 18 | 0 |
| Electrical engineering | 50 | 0 |
| Mathematics | 23 | 0 |
| Physics | 40 | 1 |
| Unit converter | 1 | 0 |

| Section | Calculator | Status | Templates |
|---|---|---|---|
| Materials | `materials.library` | Not started |  |
| Materials | `materials.rambergOsgood` | Not started |  |
| Materials | `materials.periodicTable` | Not started |  |
| Materials | `materials.selection` | Not started |  |
| Materials | `materials.composites` | Not started |  |
| Materials | `materials.creepRate` | Not started |  |
| Materials | `materials.diffusion` | Not started |  |
| Materials | `materials.weibull` | Not started |  |
| Materials | `materials.corrosion` | Not started |  |
| Materials | `materials.pitting` | Not started |  |
| Geometry | `geometry.area` | Not started |  |
| Geometry | `geometry.secondMoment` | Templates | `geometry.sections.iSection`, `geometry.sections.rectangularTube` |
| Geometry | `geometry.sectionModulus` | Not started |  |
| Geometry | `geometry.sectionStiffness` | Not started |  |
| Geometry | `geometry.solids` | Not started |  |
| Geometry | `geometry.compositeSolids` | Not started |  |
| Mechanical engineering | `mechanical.staticFailure` | Templates | `mechanical.staticFailure.brittle`, `mechanical.staticFailure.planeStress`, `mechanical.staticFailure.triaxial` |
| Mechanical engineering | `mechanical.fatigue` | Templates | `mechanical.fatigue.enduranceLimit`, `mechanical.fatigue.fluctuating`, `mechanical.fatigue.life`, `mechanical.fatigue.notchFactor` |
| Mechanical engineering | `mechanical.stressConcentration` | Templates | `mechanical.stressConcentration.plateHole`, `mechanical.stressConcentration.shaftGroove`, `mechanical.stressConcentration.shaftShoulder` |
| Mechanical engineering | `mechanical.curvedBeams` | Templates | `mechanical.curvedBeams.rectangle`, `mechanical.curvedBeams.round`, `mechanical.curvedBeams.trapezoid` |
| Mechanical engineering | `mechanical.residualStresses` | Templates | `mechanical.residualStresses.rectangularBeam`, `mechanical.residualStresses.shaftTorsion` |
| Mechanical engineering | `mechanical.fractureMechanics` | Templates | `mechanical.fracture.leakBeforeBreak`, `mechanical.fracture.parisLife` |
| Mechanical engineering | `mechanical.kIcSolver` | Templates | `mechanical.fracture.criticalCrack`, `mechanical.fracture.thickPlateSurfaceCrack` |
| Mechanical engineering | `mechanical.reliability` | Templates | `mechanical.reliability.allowableStress`, `mechanical.reliability.designFactor`, `mechanical.reliability.interference` |
| Mechanical engineering | `mechanical.strainGauges` | Templates | `mechanical.strainGauges.bridge`, `mechanical.strainGauges.quarterBridge`, `mechanical.strainGauges.shaftTorque` |
| Mechanical engineering | `mechanical.strainRosettes` | Templates | `mechanical.strainRosettes.equiangular`, `mechanical.strainRosettes.rectangular` |
| Mechanical engineering | `mechanical.threads` | Templates | `mechanical.threads.basicSize` |
| Mechanical engineering | `mechanical.threadsAnySize` | Templates | `mechanical.threads.strength` |
| Mechanical engineering | `mechanical.powerScrews` | Templates | `mechanical.powerScrews.stresses`, `mechanical.powerScrews.torque` |
| Mechanical engineering | `mechanical.boltTorqueSimple` | Templates | `mechanical.boltTorque.simple` |
| Mechanical engineering | `mechanical.boltedJoint` | Templates | `mechanical.boltedJoint.fatigue`, `mechanical.boltedJoint.nasaMargins`, `mechanical.boltedJoint.tension` |
| Mechanical engineering | `mechanical.vdi2230` | Templates | `mechanical.vdi2230.loadFactor`, `mechanical.vdi2230.preloadChecks` |
| Mechanical engineering | `mechanical.boltGroups` | Templates | `mechanical.boltGroups.eccentricShear`, `mechanical.boltGroups.tension` |
| Mechanical engineering | `mechanical.rivetedJoints` | Templates | `mechanical.rivetedJoints.shear` |
| Mechanical engineering | `mechanical.weldGroup` | Templates | `mechanical.weldGroup.circle`, `mechanical.weldGroup.rectangle` |
| Mechanical engineering | `mechanical.weldCapacity` | Templates | `mechanical.weldCapacity.fillet`, `mechanical.weldCapacity.filletFatigue` |
| Mechanical engineering | `mechanical.intermittentWelds` | Templates | `mechanical.intermittentWelds.lengthPitch` |
| Mechanical engineering | `mechanical.adhesiveJoints` | Templates | `mechanical.adhesiveJoints.doubleLap`, `mechanical.adhesiveJoints.singleLap` |
| Mechanical engineering | `mechanical.compressionSprings` | Templates | `mechanical.compressionSprings.fatigue`, `mechanical.compressionSprings.static` |
| Mechanical engineering | `mechanical.extensionSprings` | Templates | `mechanical.extensionSprings.fatigue`, `mechanical.extensionSprings.static` |
| Mechanical engineering | `mechanical.torsionSprings` | Templates | `mechanical.torsionSprings.helical` |
| Mechanical engineering | `mechanical.discSprings` | Templates | `mechanical.discSprings.single`, `mechanical.discSprings.stack` |
| Mechanical engineering | `mechanical.waveSprings` | Templates | `mechanical.waveSprings.smalley` |
| Mechanical engineering | `mechanical.conicalSprings` | Templates | `mechanical.conicalSprings.uniformPitch` |
| Mechanical engineering | `mechanical.flatSprings` | Templates | `mechanical.flatSprings.table19` |
| Mechanical engineering | `mechanical.spiralSprings` | Templates | `mechanical.spiralSprings.uniformBending` |
| Mechanical engineering | `mechanical.leafSprings` | Templates | `mechanical.leafSprings.multiLeaf` |
| Mechanical engineering | `mechanical.torsionBars` | Templates | `mechanical.torsionBars.rectangle`, `mechanical.torsionBars.round` |
| Mechanical engineering | `mechanical.pressureVessels` | Templates | `mechanical.pressureVessels.thickCylinder`, `mechanical.pressureVessels.thickSphere`, `mechanical.pressureVessels.thinCone`, `mechanical.pressureVessels.thinCylinder`, `mechanical.pressureVessels.thinSphere` |
| Mechanical engineering | `mechanical.pressureHeads` | Templates | `mechanical.pressureHeads.ellipsoidal`, `mechanical.pressureHeads.hemispherical`, `mechanical.pressureHeads.torispherical` |
| Mechanical engineering | `mechanical.rectangularPlates` | Templates | `mechanical.rectangularPlates.coefficients`, `mechanical.rectangularPlates.simplySupported` |
| Mechanical engineering | `mechanical.stiffenedPlates` | Templates | `mechanical.stiffenedPlates.frame`, `mechanical.stiffenedPlates.pitch`, `mechanical.stiffenedPlates.stiffener` |
| Mechanical engineering | `mechanical.rollingBearings` | Templates | `mechanical.rollingBearings.dutyCycle`, `mechanical.rollingBearings.life`, `mechanical.rollingBearings.lubrication`, `mechanical.rollingBearings.taperedPair` |
| Mechanical engineering | `mechanical.hertzContact` | Templates | `mechanical.hertzContact.cylinders`, `mechanical.hertzContact.elliptical`, `mechanical.hertzContact.spheres`, `mechanical.hertzContact.surfaceFatigue` |
| Mechanical engineering | `mechanical.wear` | Templates | `mechanical.wear.archard`, `mechanical.wear.pinOnDisk` |
| Mechanical engineering | `mechanical.bearingFits` | Templates | `mechanical.bearingFits.seats` |
| Mechanical engineering | `mechanical.journalBearings` | Templates | `mechanical.journalBearings.lengthRatio`, `mechanical.journalBearings.petroff`, `mechanical.journalBearings.pressureFed`, `mechanical.journalBearings.raimondiBoyd` |
| Mechanical engineering | `mechanical.boundaryBearings` | Templates | `mechanical.boundaryBearings.temperature`, `mechanical.boundaryBearings.wear` |
| Mechanical engineering | `mechanical.lubricantViscosity` | Templates | `mechanical.lubricantViscosity.roelands`, `mechanical.lubricantViscosity.saeGrade`, `mechanical.lubricantViscosity.saybolt`, `mechanical.lubricantViscosity.viscosityIndex`, `mechanical.lubricantViscosity.walther` |
| Mechanical engineering | `mechanical.shaftDesign` | Templates | `mechanical.shaftDesign.diameter`, `mechanical.shaftDesign.factorOfSafety`, `mechanical.shaftDesign.stiffness` |
| Mechanical engineering | `mechanical.shaftCriticalSpeed` | Templates | `mechanical.shaftCriticalSpeed.influence`, `mechanical.shaftCriticalSpeed.simplySupported` |
| Mechanical engineering | `mechanical.keysSplines` | Templates | `mechanical.keysSplines.involuteSpline`, `mechanical.keysSplines.parallelKey`, `mechanical.keysSplines.pin`, `mechanical.keysSplines.straightSpline` |
| Mechanical engineering | `mechanical.interferenceFits` | Templates | `mechanical.interferenceFits.pressFit` |
| Mechanical engineering | `mechanical.couplings` | Templates | `mechanical.couplings.flangeBolts`, `mechanical.couplings.hookeJoint`, `mechanical.couplings.twoJoints` |
| Mechanical engineering | `mechanical.fits` | Templates | `mechanical.fits.iso286Formulas`, `mechanical.fits.limits` |
| Mechanical engineering | `mechanical.iso2768` | Templates | `mechanical.iso2768.general` |
| Mechanical engineering | `mechanical.gdtFrame` | Templates | `mechanical.gdtFrame.virtualSize` |
| Mechanical engineering | `mechanical.gdtSymbols` | Reference page, not converted: owner's decision |  |
| Mechanical engineering | `mechanical.gdtBonus` | Templates | `mechanical.gdtBonus.lmc`, `mechanical.gdtBonus.mmc`, `mechanical.gdtBonus.rfs` |
| Mechanical engineering | `mechanical.gdtPosition` | Templates | `mechanical.gdtPosition.coordinates`, `mechanical.gdtPosition.cylindrical`, `mechanical.gdtPosition.spherical` |
| Mechanical engineering | `mechanical.gdtFasteners` | Templates | `mechanical.gdtFasteners.coaxial`, `mechanical.gdtFasteners.fixed`, `mechanical.gdtFasteners.fixedTilt`, `mechanical.gdtFasteners.floating` |
| Mechanical engineering | `mechanical.toleranceStack` | Templates | `mechanical.toleranceStack.worstCaseRss` |
| Mechanical engineering | `mechanical.diskClutches` | Templates | `mechanical.diskClutches.annularPad`, `mechanical.diskClutches.buttonPad`, `mechanical.diskClutches.plate` |
| Mechanical engineering | `mechanical.coneClutches` | Templates | `mechanical.coneClutches.capacity` |
| Mechanical engineering | `mechanical.drumBrakes` | Templates | `mechanical.drumBrakes.longShoe`, `mechanical.drumBrakes.pivotedShoe`, `mechanical.drumBrakes.shortShoe` |
| Mechanical engineering | `mechanical.bandBrakes` | Templates | `mechanical.bandBrakes.fromForce`, `mechanical.bandBrakes.fromPressure` |
| Mechanical engineering | `mechanical.brakeEnergy` | Templates | `mechanical.brakeEnergy.clutchEngagement`, `mechanical.brakeEnergy.movingStop`, `mechanical.brakeEnergy.repeatedStops`, `mechanical.brakeEnergy.rotatingStop` |
| Mechanical engineering | `mechanical.flywheels` | Templates | `mechanical.flywheels.energyFluctuation`, `mechanical.flywheels.punchPress` |
| Mechanical engineering | `mechanical.flatBelts` | Templates | `mechanical.flatBelts.capacity`, `mechanical.flatBelts.design`, `mechanical.flatBelts.metal` |
| Mechanical engineering | `mechanical.vBelts` | Templates | `mechanical.vBelts.drive` |
| Mechanical engineering | `mechanical.timingBelts` | Templates | `mechanical.timingBelts.drive` |
| Mechanical engineering | `mechanical.rollerChains` | Templates | `mechanical.rollerChains.drive` |
| Mechanical engineering | `mechanical.wireRope` | Templates | `mechanical.wireRope.hoist` |
| Mechanical engineering | `mechanical.gearGeometry` | Templates | `mechanical.gearGeometry.centreDistance`, `mechanical.gearGeometry.external` |
| Mechanical engineering | `mechanical.gearRating` | Templates | `mechanical.gearRating.agma`, `mechanical.gearRating.juvinall` |
| Mechanical engineering | `mechanical.bevelGears` | Templates | `mechanical.bevelGears.geometry`, `mechanical.bevelGears.rating` |
| Mechanical engineering | `mechanical.wormGears` | Templates | `mechanical.wormGears.agma` |
| Mechanical engineering | `mechanical.gearTrains` | Templates | `mechanical.gearTrains.differential`, `mechanical.gearTrains.ordinary`, `mechanical.gearTrains.planetary` |
| Mechanical engineering | `mechanical.toothThickness` | Templates | `mechanical.toothThickness.chordal`, `mechanical.toothThickness.overPins`, `mechanical.toothThickness.span` |
| Mechanical engineering | `fluids.depthPressure` | Templates | `fluids.depthPressure.depth`, `fluids.depthPressure.gate`, `fluids.depthPressure.pressure` |
| Mechanical engineering | `fluids.buoyancy` | Templates | `fluids.buoyancy.floatingBox`, `fluids.buoyancy.immersed` |
| Mechanical engineering | `fluids.drag` | Templates | `fluids.drag.body`, `fluids.drag.cylinder`, `fluids.drag.sphereTerminal` |
| Mechanical engineering | `fluids.pipeFlow` | Templates | `fluids.pipes.darcyColebrook` |
| Mechanical engineering | `dynamics.singleDof` | Templates | `dynamics.singleDof.forced` |
| Mechanical engineering | `dynamics.absorber` | Templates | `dynamics.absorber.optimum`, `dynamics.absorber.response` |
| Mechanical engineering | `dynamics.naturalFrequencies` | Templates | `dynamics.naturalFrequencies.bar`, `dynamics.naturalFrequencies.beam`, `dynamics.naturalFrequencies.circularPlate`, `dynamics.naturalFrequencies.rectangularPlate`, `dynamics.naturalFrequencies.springMass` |
| Mechanical engineering | `dynamics.torsional` | Templates | `dynamics.torsional.holzer` |
| Mechanical engineering | `dynamics.criticalSpeeds` | Templates | `dynamics.criticalSpeeds.shaft` |
| Mechanical engineering | `dynamics.balancing` | Templates | `dynamics.balancing.iso1940`, `dynamics.balancing.singlePlane`, `dynamics.balancing.twoPlane` |
| Mechanical engineering | `mechanics.capstan` | Templates | `mechanics.capstan.tension`, `mechanics.capstan.wrap` |
| Mechanical engineering | `mechanics.impact` | Templates | `mechanics.impact.central` |
| Mechanical engineering | `mechanical.impactLoading` | Templates | `mechanical.impactLoading.beam`, `mechanical.impactLoading.drop`, `mechanical.impactLoading.energyCapacity`, `mechanical.impactLoading.moving`, `mechanical.impactLoading.torsion` |
| Mechanical engineering | `mechanics.gyroscope` | Templates | `mechanics.gyroscope.roll`, `mechanics.gyroscope.turning` |
| Mechanical engineering | `mechanical.oRings` | Templates | `mechanical.oRings.faceGland`, `mechanical.oRings.radialGland` |
| Mechanical engineering | `mechanical.quadRings` | Templates | `mechanical.quadRings.radialGland` |
| Mechanical engineering | `mechanical.lipSeals` | Templates | `mechanical.lipSeals.speed` |
| Mechanical engineering | `mechanical.hexScrews` | Templates | `mechanical.tables.asme.hexBolt`, `mechanical.tables.asme.hexCapScrew`, `mechanical.tables.iso.hexBolt` |
| Mechanical engineering | `mechanical.heavyHexScrews` | Templates | `mechanical.tables.asme.heavyHexBolt`, `mechanical.tables.asme.heavyHexScrew`, `mechanical.tables.asme.structuralBolt`, `mechanical.tables.iso.heavyHexBolt` |
| Mechanical engineering | `mechanical.socketHeadScrews` | Templates | `mechanical.tables.asme.socketHeadCapScrew`, `mechanical.tables.iso.socketHeadCapScrew` |
| Mechanical engineering | `mechanical.lowHeadScrews` | Templates | `mechanical.tables.asme.lowHeadCapScrew`, `mechanical.tables.din.lowHeadCapScrew` |
| Mechanical engineering | `mechanical.buttonHeadScrews` | Templates | `mechanical.tables.asme.buttonHeadCapScrew`, `mechanical.tables.iso.buttonHeadCapScrew`, `mechanical.tables.iso.flangedButtonHeadCapScrew` |
| Mechanical engineering | `mechanical.flatHeadScrews` | Templates | `mechanical.tables.asme.flatHeadCapScrew`, `mechanical.tables.iso.flatHeadCapScrew` |
| Mechanical engineering | `mechanical.hexNuts` | Templates | `mechanical.tables.asme.heavyHexJamNut`, `mechanical.tables.asme.heavyHexNut`, `mechanical.tables.asme.hexJamNut`, `mechanical.tables.asme.hexNut`, `mechanical.tables.iso.heavyHexNut`, `mechanical.tables.iso.hexNut`, `mechanical.tables.iso.thinNut` |
| Mechanical engineering | `mechanical.washers` | Templates | `mechanical.tables.asme.metricWasherNarrow`, `mechanical.tables.asme.metricWasherRegular`, `mechanical.tables.asme.metricWasherWide`, `mechanical.tables.asme.washerTypeAN`, `mechanical.tables.asme.washerTypeAOther`, `mechanical.tables.asme.washerTypeAW`, `mechanical.tables.asme.washerTypeBN`, `mechanical.tables.asme.washerTypeBR`, `mechanical.tables.asme.washerTypeBW`, `mechanical.tables.astm.f436Washer`, `mechanical.tables.iso.washerExtraLarge`, `mechanical.tables.iso.washerLarge`, `mechanical.tables.iso.washerNormal`, `mechanical.tables.iso.washerSmall` |
| Mechanical engineering | `mechanical.boltGrades` | Templates | `mechanical.tables.boltGrades.astmPressure`, `mechanical.tables.boltGrades.astmStructural`, `mechanical.tables.boltGrades.f593`, `mechanical.tables.boltGrades.iso898`, `mechanical.tables.boltGrades.sae` |
| Mechanical engineering | `mechanical.discSpringSizes` | Templates | `mechanical.tables.en16983.seriesA`, `mechanical.tables.en16983.seriesB`, `mechanical.tables.en16983.seriesC` |
| Mechanical engineering | `mechanical.pipeSizes` | Templates | `mechanical.tables.pipe.stainless`, `mechanical.tables.pipe.steel`, `mechanical.tables.pipe.steelLarge` |
| Machining | `mechanical.turning` | Not started |  |
| Machining | `mechanical.milling` | Not started |  |
| Machining | `mechanical.drilling` | Not started |  |
| Machining | `mechanical.chipThinning` | Not started |  |
| Machining | `mechanical.speedTables` | Not started |  |
| Machining | `machining.gcodeInterpreter` | Not started |  |
| Machining | `machining.gcodeDictionary` | Not started |  |
| Structural engineering | `structural.loadCombinations` | Templates | `structural.loads.combinations` |
| Structural engineering | `structural.snow` | Templates | `structural.snow.roof`, `structural.snow.stepDrift`, `structural.tables.asce7.importance`, `structural.tables.asce7.snowExposure`, `structural.tables.asce7.snowThermal` |
| Structural engineering | `structural.seismic` | Templates | `structural.seismic.baseShear`, `structural.seismic.verticalDistribution`, `structural.tables.asce7.periodCu`, `structural.tables.asce7.periodParameters`, `structural.tables.asce7.seismicSystems`, `structural.tables.asce7.siteFa`, `structural.tables.asce7.siteFv` |
| Structural engineering | `structural.beam` | Templates | `structural.beams.cantileverPointLoad`, `structural.beams.cantileverUniform`, `structural.beams.fixedPointLoad`, `structural.beams.fixedUniform`, `structural.beams.proppedPointLoad`, `structural.beams.proppedUniform`, `structural.beams.simplePointLoad`, `structural.beams.simpleTriangular`, `structural.beams.simpleUniformDiagrams`, `structural.beams.twoSpanUniform` |
| Structural engineering | `structural.frames` | Templates | `structural.frames.portalFixed`, `structural.frames.portalPinned` |
| Structural engineering | `structural.catenary` | Templates | `structural.catenary.horizontalTension`, `structural.catenary.lengthGiven`, `structural.catenary.sagGiven` |
| Structural engineering | `structural.sectionBuilder` | Templates | `structural.sections.builtUpRectangles`, `structural.sections.iBeamUnequalFlanges` |
| Structural engineering | `structural.tensileFrequency` | Templates | `structural.tensile.modesAtTension`, `structural.tensile.tensionFromFrequency`, `structural.tensile.tensionFromFrequencyClamped` |
| Structural engineering | `structural.tensileElongation` | Templates | `structural.tensile.elongation` |
| Structural engineering | `structural.bolts` | Templates | `structural.bolts.shearTension` |
| Structural engineering | `structural.boltedPlate` | Templates | `structural.boltedPlate.tension` |
| Structural engineering | `structural.filletWelds` | Templates | `structural.welds.filletStrength` |
| Structural engineering | `structural.grooveWelds` | Templates | `structural.grooveWelds.cjp`, `structural.grooveWelds.flare`, `structural.grooveWelds.pjp` |
| Structural engineering | `structural.basePlates` | Templates | `structural.basePlates.concentric` |
| Structural engineering | `structural.plateBuckling` | Templates | `structural.plateBuckling.compressionSS`, `structural.plateBuckling.general`, `structural.plateBuckling.shear` |
| Structural engineering | `structural.columnBuckling` | Templates | `mechanical.columns.eulerJohnson` |
| Structural engineering | `structural.columnEccentric` | Templates | `structural.column.secant` |
| Structural engineering | `structural.columnSlenderness` | Templates | `structural.column.alignmentK`, `structural.column.slenderness` |
| Structural engineering | `structural.steelMember` | Templates | `structural.steel.flexureF2` |
| Structural engineering | `structural.stainlessMember` | Templates | `structural.stainless.compressionE3`, `structural.stainless.compressionRectHSS`, `structural.stainless.flexureF2`, `structural.stainless.shearWeb` |
| Structural engineering | `structural.aluminumMember` | Templates | `structural.aluminum.compressionIBeam`, `structural.aluminum.compressionRoundTube`, `structural.aluminum.flexureIBeam` |
| Structural engineering | `structural.lateralBuckling` | Templates | `structural.ltb.cbF1`, `structural.ltb.elasticMcr`, `structural.ltb.rectBarF11` |
| Structural engineering | `structural.aiscShapes` | Templates | `structural.tables.aisc.2l`, `structural.tables.aisc.c`, `structural.tables.aisc.hp`, `structural.tables.aisc.hss`, `structural.tables.aisc.hssRound`, `structural.tables.aisc.l`, `structural.tables.aisc.m`, `structural.tables.aisc.mc`, `structural.tables.aisc.mt`, `structural.tables.aisc.pipe`, `structural.tables.aisc.s`, `structural.tables.aisc.st`, `structural.tables.aisc.w`, `structural.tables.aisc.wt` |
| Structural engineering | `structural.enShapes` | Templates | `structural.tables.en.hd`, `structural.tables.en.heA`, `structural.tables.en.heAA`, `structural.tables.en.heB`, `structural.tables.en.heM`, `structural.tables.en.heX`, `structural.tables.en.hl`, `structural.tables.en.hp`, `structural.tables.en.ipe`, `structural.tables.en.ipeA`, `structural.tables.en.ipeAA`, `structural.tables.en.ipeO`, `structural.tables.en.ipeV`, `structural.tables.en.ipn`, `structural.tables.en.l`, `structural.tables.en.lu`, `structural.tables.en.pfc`, `structural.tables.en.ub`, `structural.tables.en.uc`, `structural.tables.en.upe`, `structural.tables.en.upn` |
| Structural engineering | `structural.coldFormedShapes` | Templates | `structural.tables.sfia.hatFurringChannels`, `structural.tables.sfia.nonStructuralStuds`, `structural.tables.sfia.structuralStuds`, `structural.tables.sfia.tracks`, `structural.tables.sfia.uChannels` |
| Structural engineering | `structural.admShapes` | Templates | `structural.tables.adm.table11`, `structural.tables.adm.table12`, `structural.tables.adm.table13`, `structural.tables.adm.table14`, `structural.tables.adm.table15`, `structural.tables.adm.table16`, `structural.tables.adm.table17`, `structural.tables.adm.table18`, `structural.tables.adm.table20`, `structural.tables.adm.table21`, `structural.tables.adm.table22`, `structural.tables.adm.table23`, `structural.tables.adm.table24`, `structural.tables.adm.table4`, `structural.tables.adm.table5`, `structural.tables.adm.table6`, `structural.tables.adm.table7`, `structural.tables.adm.table8`, `structural.tables.adm.table9` |
| Structural engineering | `structural.windVelocity` | Templates | `structural.wind.topographicFactor`, `structural.wind.velocityPressure` |
| Structural engineering | `structural.windBuilding` | Templates | `structural.wind.gustRigid`, `structural.wind.mwfrsDirectional` |
| Structural engineering | `structural.windComponents` | Templates | `structural.wind.componentsLowRise` |
| Structural engineering | `structural.windOther` | Templates | `structural.wind.solidSign`, `structural.wind.structureForce`, `structural.wind.trussedTower` |
| Thermal engineering | `thermal.steamTables` | Not started |  |
| Thermal engineering | `thermal.refrigerants` | Not started |  |
| Thermal engineering | `thermal.idealGases` | Not started |  |
| Thermal engineering | `thermal.steadyFlow` | Not started |  |
| Thermal engineering | `thermal.gasProcesses` | Not started |  |
| Thermal engineering | `thermal.rankine` | Not started |  |
| Thermal engineering | `thermal.gasCycles` | Not started |  |
| Thermal engineering | `thermal.combinedCycle` | Not started |  |
| Thermal engineering | `thermal.vaporCompression` | Not started |  |
| Thermal engineering | `thermal.gasRefrigeration` | Not started |  |
| Thermal engineering | `thermal.resistance` | Not started |  |
| Thermal engineering | `thermal.conduction` | Not started |  |
| Thermal engineering | `thermal.convection` | Not started |  |
| Thermal engineering | `thermal.fins` | Not started |  |
| Thermal engineering | `thermal.radiation` | Not started |  |
| Thermal engineering | `thermal.heatExchangers` | Not started |  |
| Thermal engineering | `thermal.transient` | Not started |  |
| Thermal engineering | `thermal.thermalStress` | Not started |  |
| Electrical engineering | `electrical.ohmsLaw` | Not started |  |
| Electrical engineering | `electrical.seriesParallel` | Not started |  |
| Electrical engineering | `electrical.dividers` | Not started |  |
| Electrical engineering | `electrical.wheatstone` | Not started |  |
| Electrical engineering | `electrical.deltaWye` | Not started |  |
| Electrical engineering | `electrical.wire` | Not started |  |
| Electrical engineering | `electrical.resistor` | Not started |  |
| Electrical engineering | `electrical.cableAmpacity` | Not started |  |
| Electrical engineering | `electrical.ampacity` | Not started |  |
| Electrical engineering | `electrical.pcbHeater` | Not started |  |
| Electrical engineering | `electrical.resistorCodes` | Not started |  |
| Electrical engineering | `electrical.jouleHeating` | Not started |  |
| Electrical engineering | `electrical.acPower` | Not started |  |
| Electrical engineering | `electrical.pfCorrection` | Not started |  |
| Electrical engineering | `electrical.waveforms` | Not started |  |
| Electrical engineering | `electrical.decibels` | Not started |  |
| Electrical engineering | `electrical.battery` | Not started |  |
| Electrical engineering | `electrical.creepage` | Not started |  |
| Electrical engineering | `electrical.insulationThickness` | Not started |  |
| Electrical engineering | `electrical.wireInductance` | Not started |  |
| Electrical engineering | `electrical.loopInductance` | Not started |  |
| Electrical engineering | `electrical.coilInductance` | Not started |  |
| Electrical engineering | `electrical.mutualInductance` | Not started |  |
| Electrical engineering | `electrical.spiralInductance` | Not started |  |
| Electrical engineering | `electrical.inductorEnergy` | Not started |  |
| Electrical engineering | `electrical.induction` | Not started |  |
| Electrical engineering | `electrical.permanentMagnet` | Not started |  |
| Electrical engineering | `electrical.coilForces` | Not started |  |
| Electrical engineering | `electrical.magneticCircuits` | Not started |  |
| Electrical engineering | `electrical.conductorField` | Not started |  |
| Electrical engineering | `electrical.loopField` | Not started |  |
| Electrical engineering | `electrical.solenoidField` | Not started |  |
| Electrical engineering | `electrical.chipPackages` | Not started |  |
| Electrical engineering | `electrical.sotPackages` | Not started |  |
| Electrical engineering | `electrical.soPackages` | Not started |  |
| Electrical engineering | `electrical.qfpPackages` | Not started |  |
| Electrical engineering | `electrical.qfnPackages` | Not started |  |
| Electrical engineering | `electrical.powerPackages` | Not started |  |
| Electrical engineering | `electrical.dipPackages` | Not started |  |
| Electrical engineering | `electrical.toPackages` | Not started |  |
| Electrical engineering | `electrical.axialPackages` | Not started |  |
| Electrical engineering | `electrical.radialPackages` | Not started |  |
| Electrical engineering | `electrical.ledZener` | Not started |  |
| Electrical engineering | `electrical.lm317` | Not started |  |
| Electrical engineering | `electrical.timer555` | Not started |  |
| Electrical engineering | `electrical.opAmp` | Not started |  |
| Electrical engineering | `electrical.reactive` | Not started |  |
| Electrical engineering | `electrical.rlcCircuits` | Not started |  |
| Electrical engineering | `electrical.adc` | Not started |  |
| Electrical engineering | `electrical.traceWidth` | Not started |  |
| Mathematics | `math.rpl` | Not started |  |
| Mathematics | `math.grapher` | Not started |  |
| Mathematics | `math.dataPlot` | Not started |  |
| Mathematics | `math.calculator` | Not started |  |
| Mathematics | `math.numbers` | Not started |  |
| Mathematics | `math.combinatorics` | Not started |  |
| Mathematics | `math.polynomial` | Not started |  |
| Mathematics | `math.solve` | Not started |  |
| Mathematics | `math.interpolate` | Not started |  |
| Mathematics | `math.coordinates` | Not started |  |
| Mathematics | `math.segment` | Not started |  |
| Mathematics | `math.triangle` | Not started |  |
| Mathematics | `math.trig` | Not started |  |
| Mathematics | `math.calculus` | Not started |  |
| Mathematics | `math.series` | Not started |  |
| Mathematics | `math.matrix` | Not started |  |
| Mathematics | `math.vectors` | Not started |  |
| Mathematics | `math.ode` | Not started |  |
| Mathematics | `math.linearOde` | Not started |  |
| Mathematics | `math.heatEquation` | Not started |  |
| Mathematics | `math.waveEquation` | Not started |  |
| Mathematics | `math.laplaceEquation` | Not started |  |
| Mathematics | `math.fourierSeries` | Not started |  |
| Physics | `physics.kinematics` | Templates | `physics.kinematics.fromDistance`, `physics.kinematics.fromTime` |
| Physics | `physics.projectile` | Not started |  |
| Physics | `physics.circular` | Not started |  |
| Physics | `physics.gravitation` | Not started |  |
| Physics | `physics.oscillators` | Not started |  |
| Physics | `physics.standingWaves` | Not started |  |
| Physics | `physics.sound` | Not started |  |
| Physics | `physics.heat` | Not started |  |
| Physics | `physics.heatTransfer` | Not started |  |
| Physics | `physics.idealGas` | Not started |  |
| Physics | `physics.heatEngines` | Not started |  |
| Physics | `physics.kineticTheory` | Not started |  |
| Physics | `physics.entropy` | Not started |  |
| Physics | `physics.phaseChangeCalorimetry` | Not started |  |
| Physics | `physics.expansionFits` | Not started |  |
| Physics | `physics.realEngines` | Not started |  |
| Physics | `physics.charges` | Not started |  |
| Physics | `physics.pointChargePotential` | Not started |  |
| Physics | `physics.capacitor` | Not started |  |
| Physics | `physics.chargeDistributions` | Not started |  |
| Physics | `physics.capacitance` | Not started |  |
| Physics | `physics.imageCharges` | Not started |  |
| Physics | `physics.dielectrics` | Not started |  |
| Physics | `physics.particleInField` | Not started |  |
| Physics | `physics.currentResistance` | Not started |  |
| Physics | `physics.magneticForces` | Not started |  |
| Physics | `physics.ampereLaw` | Not started |  |
| Physics | `physics.motorsLc` | Not started |  |
| Physics | `physics.emWaves` | Not started |  |
| Physics | `physics.refraction` | Not started |  |
| Physics | `physics.lenses` | Not started |  |
| Physics | `physics.interference` | Not started |  |
| Physics | `physics.relativity` | Not started |  |
| Physics | `physics.photons` | Not started |  |
| Physics | `physics.nuclear` | Not started |  |
| Physics | `physics.blackbody` | Not started |  |
| Physics | `physics.quantumWells` | Not started |  |
| Physics | `physics.hydrogen` | Not started |  |
| Physics | `physics.molecules` | Not started |  |
| Physics | `physics.nuclearReactions` | Not started |  |
| Unit converter | `units.converter` | Not started |  |

<!-- status:end -->
