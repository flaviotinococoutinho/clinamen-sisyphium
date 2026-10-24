#!/usr/bin/env python3
"""Propose integer rendering costs from paired measurements; never edit source."""
import argparse
import csv
import json
from pathlib import Path
import sys
import unittest

FEATURES = ('stratum', 'caelum', 'margo', 'annales', 'personae', 'ruinae',
            'impossibilia', 'hexagramma', 'puncta', 'littera', 'bicolor')
FIELDS = ('scenario', 'stilus', 'largura', 'altura', 'strata', 'layers', 'lines',
          'n', 'mediana_ns', 'p95_ns', 'tipo', 'feature', 'quantity', 'base_ns',
          'delta_ns', 'pair')


def normalize(base, delta, quantity=1):
    """Divide by the number added before ceiling; retain only positive increments."""
    if base <= 0 or quantity <= 0:
        raise ValueError('base and quantity must be positive')
    denominator = base * quantity
    return (10 * max(0, delta) + denominator - 1) // denominator


def read_rows(path):
    with path.open(newline='', encoding='utf-8') as source:
        reader = csv.DictReader(source, delimiter='\t')
        if not set(FIELDS).issubset(reader.fieldnames or ()):
            raise ValueError('missing measurement column')
        rows = []
        for row in reader:
            for field in FIELDS:
                if field not in ('scenario', 'stilus', 'tipo', 'feature', 'pair'):
                    row[field] = int(row[field])
            if row['mediana_ns'] <= 0 or row['p95_ns'] < row['mediana_ns']:
                raise ValueError('invalid timing summary')
            if row['tipo'] == 'pair':
                row['coefficient'] = normalize(row['base_ns'], row['delta_ns'], row['quantity'])
            elif row['tipo'] != 'full':
                raise ValueError('unknown measurement kind')
            rows.append(row)
    return rows


def median(values):
    values = sorted(values)
    return values[len(values) // 2]


def propose(rows):
    """Use the maximum normalized paired median, retaining dispersion and fit evidence."""
    expected = {(s, t, w, h) for s in ('vigilia', 'acedia', 'somnium', 'quies', 'littera')
                for t in ('creta', 'musivum', 'littera', 'bicolor')
                for w, h in ((320, 180), (1920, 1080))}
    full = [r for r in rows if r['tipo'] == 'full']
    key = lambda r: (r['scenario'], r['stilus'], r['largura'], r['altura'])
    if len(full) != 40 or {key(r) for r in full} != expected:
        raise ValueError('incomplete or duplicate reference matrix: require 5 x 4 x 2')
    grouped = {k: [] for k in expected}
    for row in rows:
        if key(row) not in expected:
            raise ValueError('unexpected scenario, style or dimensions')
        if row['n'] != (1000 if row['largura'] == 320 else 100):
            raise ValueError('incorrect sample count')
        if row['tipo'] == 'pair':
            grouped[key(row)].append(row)
    for k, group in grouped.items():
        quantities = {(r['feature'], r['quantity']) for r in group}
        wanted = {('stratum', q) for q in (1, 2, 3)} | {('annales', q) for q in range(1, 7)}
        wanted |= {(f, 1) for f in FEATURES[1:9] if f != 'annales'}
        if k[1] in ('littera', 'bicolor'):
            wanted.add((k[1], 1))
        if quantities != wanted or len(group) != len(wanted):
            raise ValueError(f'incomplete or duplicate component pairs: {k}')
    costs = {'base': 10, 'creta': 0, 'musivum': 0}
    evidence = {}
    for feature in FEATURES:
        samples = [r for r in rows if r['tipo'] == 'pair' and r['feature'] == feature]
        winner = max(samples, key=lambda r: r['coefficient'])
        costs[feature] = winner['coefficient']
        coefficients = [r['coefficient'] for r in samples]
        evidence[feature] = {'min': min(coefficients), 'median': median(coefficients),
                             'max': costs[feature], 'source': winner}
    if any(c > 255 for c in costs.values()):
        raise ValueError('coefficient exceeds packed byte; representation review required')
    predictions = []
    for row in full:
        reference_style = 'musivum' if row['stilus'] == 'musivum' else 'creta'
        reference_key = (row['scenario'], reference_style, row['largura'], row['altura'])
        base_ns = median([r['base_ns'] for r in grouped[reference_key] if r['feature'] == 'stratum'])
        units = 10 + costs['stratum'] * (row['strata'] - 1) + costs[row['stilus']]
        for bit, feature in enumerate(FEATURES[1:9]):
            if row['layers'] & (1 << bit):
                units += costs[feature] * (row['lines'] if feature == 'annales' else 1)
        predicted = base_ns * units // 10
        observed = row['mediana_ns']
        anchor = median([r['base_ns'] for r in grouped[(row['scenario'], 'creta', 320, 180)]
                         if r['feature'] == 'stratum'])
        scale = max(1, min(row['largura'] // 320, row['altura'] // 180)) if row['stilus'] == 'musivum' else 1
        area = (row['largura'] // scale) * (row['altura'] // scale)
        global_prediction = anchor * units * area // (10 * 57600)
        signed_basis_points = (predicted - observed) * 10000 // observed
        predictions.append({'scenario': row['scenario'], 'stilus': row['stilus'],
                            'width': row['largura'], 'height': row['altura'],
                            'observed_ns': observed, 'predicted_ns': predicted,
                            'deviation_basis_points': signed_basis_points,
                            'requires_explanation': abs(predicted - observed) * 4 > observed,
                            'area_model_predicted_ns': global_prediction,
                            'area_model_deviation_basis_points': (global_prediction - observed) * 10000 // observed,
                            'area_model_requires_explanation': abs(global_prediction - observed) * 4 > observed})
    return {'costs': costs, 'dispersion': evidence, 'predictions': predictions,
            'method': 'maximum ceil(10 * positive paired median delta / (paired baseline median * quantity)); base=10',
            'physical_energy_measured': False}


class NormalizationTests(unittest.TestCase):
    def test_calibration_integer_normalization(self):
        self.assertEqual(normalize(1000, 250), 3)
        self.assertEqual(normalize(1000, -250), 0)
        self.assertEqual(normalize(1000, 250, 3), 1)
        self.assertEqual(normalize(1000, 601, 3), 3)
        with self.assertRaises(ValueError):
            normalize(0, 250)
        with self.assertRaises(ValueError):
            normalize(1000, 250, 0)

    def test_missing_column(self):
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'bad.tsv'
            path.write_text('scenario\tn\nvigilia\t1000\n', encoding='utf-8')
            with self.assertRaises(ValueError):
                read_rows(path)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('tsv', nargs='?', type=Path)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        unittest.main(argv=[sys.argv[0]], exit=True)
    if args.tsv is None:
        parser.error('TSV path is required')
    try:
        print(json.dumps(propose(read_rows(args.tsv)), indent=2))
    except (ValueError, OSError, KeyError) as error:
        parser.exit(1, f'calibration: {error}\n')
