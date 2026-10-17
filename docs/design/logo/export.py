"""SVG exports. resvg 0.47.0 + Pillow 12.3.0, design-time only.

Identity masters remain paths only. Social compositions additionally reference
committed original raster scenes, then use a deterministic PNG palette export.
"""
import argparse
from pathlib import Path
import struct
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from PIL import Image, __version__ as pillow_version
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
      ('social-preview-variants/threshold.svg',HERE/'social-preview-variants/threshold.png',1280,640),
      ('social-preview-variants/atlas.svg',HERE/'social-preview-variants/atlas.png',1280,640),
      ('contact-sheet.svg',HERE/'contact-sheet.png',900,720)]
SOCIAL={'social-preview.svg','social-preview-variants/threshold.svg','social-preview-variants/atlas.svg'}
BRAND_COLORS=['F8EAD5','B88960','52272B','FFD078','120C0D']

def validate_social(master):
    tree=ET.parse(master)
    images=[]
    for el in tree.iter():
        tag=el.tag.rsplit('}',1)[-1]
        assert tag in {'svg','g','path','image','defs','linearGradient','stop'},(master,tag)
        assert not any('font' in k or ('url(' in v and v!='url(#copy-fade)') for k,v in el.attrib.items()),master
        if tag=='image':
            source=(master.parent/el.attrib['href']).resolve()
            assert source.parent==HERE/'social-preview-layers',source
            assert source.name in {'rose-wall.png','threshold.png','atlas.png'},source
            assert source.is_file(),source
            images.append(source)
        else:
            assert not any('href' in k for k in el.attrib),master
    assert len(images)==1,master
    logo=next(el for el in tree.getroot() if el.get('id')=='production-logo')
    sources=([ET.parse(HERE/'emblem.svg').getroot(),ET.parse(HERE/'horizontal.svg').getroot()[1]]
             if master.name=='social-preview.svg' else [ET.parse(HERE/'horizontal.svg').getroot()])
    sources.append(ET.parse(HERE/'full-name.svg').getroot()[1])
    assert len(logo)==len(sources)
    for group,source in zip(logo,sources):
        assert [ET.tostring(el) for el in group]==[ET.tostring(el) for el in source], 'logo contours changed'
    # Conservative contrast check across the entire tagline area, including
    # spaces; the dark wall layer is part of every composition.
    background=Image.open(images[0]).convert('RGB').resize((1280,640),Image.Resampling.LANCZOS)
    def luminance(rgb):
        linear=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
        return sum(a*b for a,b in zip(linear,(.2126,.7152,.0722)))
    wall=Image.new('RGB',background.size,'#120C0D')
    shade=next(el for el in tree.getroot() if el.get('id')=='copy-shade')
    darkened=Image.blend(background,wall,float(shade[0].get('opacity')))
    worst=max(luminance(px) for px in darkened.crop((96,424,460,508)).get_flattened_data())
    contrast=(luminance((248,234,213))+.05)/(worst+.05)
    assert contrast>=4.5,(master,contrast)
    print(f'{master.name}: tagline contrast >= {contrast:.2f}:1; 34 px / 17 px at 50%')

def main():
    p=argparse.ArgumentParser(); p.add_argument('--resvg',default='resvg');p.add_argument('--check',action='store_true');args=p.parse_args()
    version=subprocess.check_output([args.resvg,'--version'],text=True).strip()
    assert version=='0.47.0',version
    assert pillow_version=='12.3.0',pillow_version
    # D-023: production must retain the owner-selected Rose Compass contours.
    assert (HERE/'emblem.svg').read_bytes()==(HERE.parent/'logo-v2/rose-compass/emblem.svg').read_bytes()
    for master in HERE.glob('*.svg'):
        if master.name in SOCIAL:
            validate_social(master)
            continue
        tree=ET.parse(master)
        for el in tree.iter():
            assert el.tag.rsplit('}',1)[-1] in {'svg','g','path'},(master,el.tag)
            assert not any('href' in k or 'url(' in v or 'font' in k for k,v in el.attrib.items()),master
    for name in sorted(SOCIAL-{'social-preview.svg'}): validate_social(HERE/name)
    for name in ['emblem','horizontal','stacked','full-name']:
        assert (ASSETS/f'logo/{name}.svg').read_bytes()==(HERE/f'{name}.svg').read_bytes()
    with tempfile.TemporaryDirectory(prefix='ddl-logo-') as scratch:
        for src,dest,w,h in JOBS:
            out=Path(scratch)/dest.name
            subprocess.run([args.resvg,'--width',str(w),'--height',str(h),str(HERE/src),str(out)],check=True)
            if src in SOCIAL:
                # Palette encoding bounds link-card size without touching vector
                # geometry or hand-editing pixels. All sources remain committed.
                im=Image.open(out).convert('RGB')
                palette=im.quantize(colors=251,method=Image.Quantize.MEDIANCUT).getpalette()[:251*3]
                palette+=list(bytes.fromhex(''.join(BRAND_COLORS)))
                swatch=Image.new('P',(1,1));swatch.putpalette(palette)
                encoded=im.quantize(palette=swatch,dither=Image.Dither.NONE)
                # Pillow's approximate RGB lookup can choose a nearby scene
                # colour even for an exact brand swatch. Preserve flat fills.
                brand={tuple(bytes.fromhex(c)):251+i for i,c in enumerate(BRAND_COLORS)}
                indices=bytearray(encoded.tobytes())
                for i,pixel in enumerate(im.get_flattened_data()):
                    if pixel in brand: indices[i]=brand[pixel]
                encoded.frombytes(bytes(indices))
                colors=set(encoded.convert('RGB').get_flattened_data())
                assert all(tuple(bytes.fromhex(c)) in colors for c in BRAND_COLORS),'brand colour lost'
                encoded.save(out,optimize=True)
            data=out.read_bytes();assert struct.unpack('>II',data[16:24])==(w,h)
            if src in SOCIAL:
                assert len(data)<=1_000_000,'social preview exceeds 1 MB'
                assert data[25]==3,'indexed social export expected'
            else: assert data[25]==6,'RGBA export expected'
            if args.check: assert dest.read_bytes()==data,f'stale export: {dest}'
            else: dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
            print(f'{dest.relative_to(ROOT)}\t{w}x{h}\t{len(data)} bytes')
if __name__=='__main__':main()
