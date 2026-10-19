"""Render and verify the v3 contact sheets. Python stdlib + resvg 0.47.0, design time only.

--check re-renders to scratch and asserts byte equality with the committed PNGs,
after validating every SVG: allowed element types only, no raster, font, script,
filter or external references, gradients referenced only by local id, and the
production Rose Compass silhouette (D-023) as every emblem's outer contour.
"""
import argparse
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
VARIANTS = ('carved-rose', 'stained-glass', 'furnace-rose')
JOBS = [(f'{v}/contact-sheet.svg', 1440, 2528) for v in VARIANTS] + [('comparison.svg', 1200, 1080)]
ELEMENTS = {'svg', 'defs', 'g', 'path', 'linearGradient', 'radialGradient', 'stop'}
ATTRIBUTES = {'width', 'height', 'viewBox', 'transform', 'd', 'fill', 'fill-rule',
              'fill-opacity', 'id', 'gradientUnits', 'cx', 'cy', 'r', 'x1', 'y1', 'x2', 'y2',
              'offset', 'stop-color'}
SILHOUETTE = [80, 4, 176, 4, 252, 80, 252, 176, 176, 252, 80, 252, 4, 176, 4, 80]


def validate(master):
    root = ET.parse(master).getroot()
    ids = {el.get('id') for el in root.iter() if el.get('id')}
    for el in root.iter():
        tag = el.tag.rsplit('}', 1)[-1]
        assert tag in ELEMENTS, (master, tag)
        for key, value in el.attrib.items():
            assert key in ATTRIBUTES, (master, key)
            for ref in re.findall(r'url\(([^)]*)\)', value):
                assert ref.startswith('#') and ref[1:] in ids, (master, ref)
            assert 'data:' not in value and '//' not in value, (master, key)
    if master.name in ('emblem.svg', 'emblem-small.svg'):
        first = next(el for el in root.iter() if el.tag.endswith('path'))
        assert [float(v) for v in re.findall(r'-?[\d.]+', first.get('d'))] == SILHOUETTE, master
    return master.stat().st_size


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--resvg', default='resvg')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    version = subprocess.check_output([args.resvg, '--version'], text=True).strip()
    assert version == '0.47.0', version
    for master in sorted(HERE.rglob('*.svg')):
        size = validate(master)
        if master.name != 'contact-sheet.svg' and master.parent != HERE:
            print(f'{master.relative_to(HERE)}\tclean vector\t{size:,} bytes')
    with tempfile.TemporaryDirectory(prefix='ddl-logo-v3-') as scratch:
        for source, w, h in JOBS:
            output = Path(scratch) / source.replace('/', '-').replace('.svg', '.png')
            subprocess.run([args.resvg, str(HERE / source), str(output)], check=True)
            data = output.read_bytes()
            assert struct.unpack('>II', data[16:24]) == (w, h), source
            target = (HERE / source).with_suffix('.png')
            if args.check:
                assert target.read_bytes() == data, f'stale export: {target}'
            else:
                target.write_bytes(data)
            print(f'{target.relative_to(HERE)}\t{w}x{h}\t{len(data):,} bytes')


if __name__ == '__main__':
    main()
