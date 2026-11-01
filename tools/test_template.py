#!/usr/bin/env python3
"""Check the real bundle's manifest against its files; no Horizon/assets required.

The helpers name game types, so they compile only in a native build on the pinned Horizon
(`horizon-launcher native-build mixins/create_horizon`). This catches what a build would
otherwise be the first to say: a missing file, a pin that disagrees, a binding that isn't a
parameter of its helper.
"""
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parent.parent


def parameters(helper):
    """The parameter names of the one function a helper file defines."""
    functions = re.findall(r'^fn \w+\s*\((.*?)\)\s*\{', helper, re.M | re.S)
    assert len(functions) == 1, 'a helper defines one function'
    names, depth, start = [], 0, 0
    for at, char in enumerate(functions[0] + ','):
        depth += char in '<(['
        depth -= char in '>)]'
        if char == ',' and depth == 0:
            name = functions[0][start:at].split(':')[0].strip()
            if name:
                names.append(name)
            start = at + 1
    return names


def main():
    compat = json.loads((ROOT/'compatibility.json').read_text())
    manifests = list((ROOT/'mixins').glob('*/horizon-mixin.toml'))
    assert len(manifests) == 1, 'this test expects one bundle'
    bundle = manifests[0].parent
    manifest = tomllib.loads(manifests[0].read_text())
    assert bundle.name == manifest['mod']['id']
    assert manifest['compatibility']['horizon_revision'] == compat['horizon_revision']
    for module in manifest['module']:
        assert (bundle/module['source']).is_file(), module['source']
    for crate in manifest.get('crate', []):
        assert (bundle/crate['path']/'Cargo.toml').is_file(), crate['path']
    injections = manifest['injection']
    assert len({i['id'] for i in injections}) == len(injections), 'injection ids repeat'
    used = set()
    for injection in injections:
        assert 'hook' in injection and 'hook_version' in injection, injection['id']
        helper = bundle/injection['helper']
        assert helper.is_file(), injection['helper']
        used.add(helper.name)
        names = [b['name'] for b in injection.get('bindings', [])]
        assert parameters(helper.read_text()) == names, injection['id']
    sources = {m['source'] for m in manifest['module']}
    leftovers = {p.name for p in bundle.glob('*.rs')} - used - sources
    assert not leftovers, f'not in the manifest: {sorted(leftovers)}'
    print(json.dumps({'status': 'ok', 'injections': len(injections)}))


if __name__ == '__main__':
    main()
