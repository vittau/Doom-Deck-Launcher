"""Render and verify concept contact sheets. Python stdlib + resvg 0.47.0."""
import argparse
from pathlib import Path
import struct
import subprocess
import tempfile
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
CONCEPTS = ('cinder-key', 'rose-compass', 'ember-folio')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--resvg', default='resvg')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    assert subprocess.check_output([args.resvg, '--version'], text=True).strip() == '0.47.0'
    with tempfile.TemporaryDirectory(prefix='ddl-concepts-') as scratch:
        for concept in CONCEPTS:
            folder = HERE / concept
            for master in folder.glob('*.svg'):
                for element in ET.parse(master).iter():
                    assert element.tag.rsplit('}', 1)[-1] in {'svg', 'g', 'path'}, master
                    assert not any('href' in k or 'url(' in v or 'font' in k
                                   for k, v in element.attrib.items()), master
            output = Path(scratch) / f'{concept}.png'
            subprocess.run([args.resvg, str(folder / 'contact-sheet.svg'), str(output)], check=True)
            data = output.read_bytes()
            assert struct.unpack('>II', data[16:24]) == (1440, 1280)
            target = folder / 'contact-sheet.png'
            if args.check:
                assert target.read_bytes() == data, f'stale export: {target}'
            else:
                target.write_bytes(data)
            print(f'{concept}: clean vector masters, 1440x1280, {len(data):,} bytes')


if __name__ == '__main__':
    main()
