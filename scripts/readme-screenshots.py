#!/usr/bin/env python3
"""Regenerate README images with the existing off-screen Slint production journeys."""
import argparse
from pathlib import Path
import os
import struct
import subprocess
import tempfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
IMAGES = {
    '010-home-continue.png': 'home.png',
    '015-discover-all-favorite.png': 'discover.png',
    '010-play-setup-auger.png': 'play-setup.png',
    '015-detail-tags.png': 'mapset-detail.png',
}


def optimized_png(data):
    """Losslessly recompress renderer scanlines; retain only required PNG chunks."""
    assert data[:8] == b'\x89PNG\r\n\x1a\n'
    chunks, compressed, offset = [], bytearray(), 8
    while offset < len(data):
        size = struct.unpack('>I', data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + size]
        if kind == b'IHDR':
            assert struct.unpack('>II', payload[:8]) == (1280, 800)
            assert payload[8:10] == bytes([8, 2]), 'expected 8-bit RGB renderer output'
            chunks.append((kind, payload))
        elif kind == b'IDAT':
            compressed.extend(payload)
        offset += size + 12
    chunks.extend([(b'IDAT', zlib.compress(zlib.decompress(compressed), 9)), (b'IEND', b'')])
    output = bytearray(data[:8])
    for kind, payload in chunks:
        output.extend(struct.pack('>I', len(payload)) + kind + payload)
        output.extend(struct.pack('>I', zlib.crc32(kind + payload)))
    return bytes(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='ddl-readme-') as scratch:
        subprocess.run(
            ['cargo', 'test', '-p', 'ddl', 'fixture_screens_navigation_and_focus_render_headlessly'],
            cwd=ROOT, env={**os.environ, 'DDL_HEADLESS_DIR': scratch}, check=True,
        )
        destination = ROOT / 'docs/screenshots'
        destination.mkdir(parents=True, exist_ok=True)
        for source, name in IMAGES.items():
            data = optimized_png((Path(scratch) / source).read_bytes())
            path = destination / name
            if args.check:
                assert path.read_bytes() == data, f'stale screenshot: {path}'
            else:
                path.write_bytes(data)
            print(f'{path.relative_to(ROOT)}: 1280x800, {len(data):,} bytes')


if __name__ == '__main__':
    main()
