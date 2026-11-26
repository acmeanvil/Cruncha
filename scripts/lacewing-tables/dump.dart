// SPDX-License-Identifier: GPL-3.0-or-later
// Dumps Lacewing Lab's standard size tables and pipe sizes as JSON (ADR-0027), for
// scripts/gen-table-templates.py. Run it with Lacewing's own package config, so nothing is
// written into that repository:
//
//   dart run --packages=$LACEWING/.dart_tool/package_config.json scripts/lacewing-tables/dump.dart > tables.json
//
// where $LACEWING is ~/Developer/Tension_Meter/App.
import 'dart:convert';

import 'package:engineering_core/piping/pipe_sizes.dart';
import 'package:engineering_core/sections/adm_shapes.dart';
import 'package:engineering_core/sections/aisc_shapes.dart';
import 'package:engineering_core/sections/en_shapes.dart';
import 'package:engineering_core/sections/sfia_shapes.dart';
import 'package:engineering_core/standards/standard_tables.dart';

Map<String, Object?> table(DimTable t) => {
      'id': t.id,
      'title': t.title,
      'standard': t.standard,
      'reference': t.reference,
      'unit': t.unit.symbol,
      'rowLabel': t.rowLabel,
      'nut': t.nutTableId,
      'notes': t.notes,
      'columns': [
        for (final c in t.columns)
          {'symbol': c.symbol, 'name': c.name, 'limit': c.limit.name, 'kind': c.kind.name, 'unit': c.unit, 'role': c.role?.name},
      ],
      'rows': [
        for (final r in t.rows) {'size': r.size, 'values': r.values, 'note': r.note},
      ],
    };

Map<String, Object?> pipe(PipeSize p) => {
      'nps': p.nps,
      'odIn': p.odM / 0.0254,
      'source': p.source.name,
      'reference': p.source.reference,
      'walls': [
        for (final w in p.walls)
          {
            'printed': w.printed,
            'unit': w.printedUnit,
            'designations': w.designations,
            'api5L': w.api5L,
            'weightLbFt': w.weightLbFt,
          },
      ],
    };

/// The four shape databases: each shape's group, name, reference and tabulated values, and
/// each field's label and unit kind (with EN's scale factor to SI).
Map<String, Object?> shapes() => {
      'aisc': {
        'source': aiscSource,
        'fields': [for (final f in SteelShape.fields) {'symbol': f.symbol, 'label': f.label, 'unit': f.unit.name}],
        'rows': [
          for (final s in SteelShape.all)
            {
              'group': s.type.code,
              'groupTitle': s.type.label,
              'name': s.label,
              'values': {for (final f in SteelShape.fields) if (s.tabulated(f.symbol) != null) f.symbol: s.tabulated(f.symbol)},
            },
        ],
      },
      'en': {
        'source': enSource,
        'families': {for (final e in enFamilies.entries) e.key: {'title': e.value.$1, 'pages': e.value.$2}},
        'fields': [
          for (final e in enFields.entries) {'symbol': e.key, 'label': e.value.$2, 'unit': e.value.$3.name, 'factor': e.value.$4},
        ],
        'rows': [
          for (final s in EuroShape.all)
            {'group': s.family, 'name': s.name, 'reference': s.reference, 'values': {for (final k in s.symbols) k: s.tabulated(k)}},
        ],
      },
      'sfia': {
        'source': sfiaSource,
        'fields': [
          for (final e in sfiaFields.entries) {'symbol': e.key, 'label': e.value.$1, 'unit': e.value.$2.name},
          for (final e in sfiaAllowables.entries) {'symbol': e.key, 'label': e.value, 'unit': 'allowable'},
        ],
        'groups': ColdFormedShape.groups,
        'rows': [
          for (final s in ColdFormedShape.all)
            {'group': s.group, 'name': s.name, 'reference': s.reference, 'values': {for (final k in s.symbols) k: s.tabulated(k)}},
        ],
      },
      'adm': {
        'source': admSource,
        'fields': [for (final e in admFields.entries) {'symbol': e.key, 'label': e.value.$1, 'unit': e.value.$2.name}],
        'tables': {for (final (n, title) in AluminumShape.tables) '$n': title},
        'rows': [
          for (final s in AluminumShape.all)
            {'group': '${s.table}', 'name': s.label, 'reference': s.reference, 'values': {for (final k in s.symbols) k: s.tabulated(k)}},
        ],
      },
    };

void main() {
  print(const JsonEncoder.withIndent(' ').convert({
    'tables': [for (final t in allStandardTables) table(t)],
    'steelPipe': [for (final p in steelPipeSizes) pipe(p)],
    'stainlessPipe': [for (final p in stainlessPipeSizes) pipe(p)],
    'shapes': shapes(),
  }));
}
