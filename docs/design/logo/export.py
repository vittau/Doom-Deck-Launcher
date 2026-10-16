"""Direct SVG exports. Python stdlib + resvg 0.47.0, design-time only."""
import argparse
from pathlib import Path
import struct
import subprocess
import tempfile
import xml.etree.ElementTree as ET
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
ASSETS=ROOT/'crates/ddl/ui/assets'
ART=HERE/'steam'
JOBS=[('emblem.svg', ASSETS/'logo/icon.png',256,256),
      ('emblem.svg',ART/'icon-256.png',256,256),('emblem.svg',ART/'icon-512.png',512,512),
      ('library-capsule.svg',ART/'library-capsule.png',600,900),
      ('wide-capsule.svg',ART/'wide-capsule.png',920,430),('hero.svg',ART/'hero.png',3840,1240),
      ('steam-logo.svg',ART/'logo.png',1280,720),
      ('social-preview.svg',HERE/'social-preview.png',1280,640),
      ('contact-sheet.svg',HERE/'contact-sheet.png',900,720)]
def main():
    p=argparse.ArgumentParser(); p.add_argument('--resvg',default='resvg');p.add_argument('--check',action='store_true');args=p.parse_args()
    version=subprocess.check_output([args.resvg,'--version'],text=True).strip()
    assert version=='0.47.0',version
    for master in HERE.glob('*.svg'):
        tree=ET.parse(master)
        for el in tree.iter():
            assert el.tag.rsplit('}',1)[-1] in {'svg','g','path'},(master,el.tag)
            assert not any('href' in k or 'url(' in v or 'font' in k for k,v in el.attrib.items()),master
    for name in ['emblem','horizontal','stacked','full-name']:
        assert (ASSETS/f'logo/{name}.svg').read_bytes()==(HERE/f'{name}.svg').read_bytes()
    with tempfile.TemporaryDirectory(prefix='ddl-logo-') as scratch:
        for src,dest,w,h in JOBS:
            out=Path(scratch)/dest.name
            subprocess.run([args.resvg,'--width',str(w),'--height',str(h),str(HERE/src),str(out)],check=True)
            data=out.read_bytes();assert struct.unpack('>II',data[16:24])==(w,h)
            assert data[25]==6,'RGBA export expected'
            if args.check: assert dest.read_bytes()==data,f'stale export: {dest}'
            else: dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
            print(f'{dest.relative_to(ROOT)}\t{w}x{h}\t{len(data)} bytes')
if __name__=='__main__':main()
