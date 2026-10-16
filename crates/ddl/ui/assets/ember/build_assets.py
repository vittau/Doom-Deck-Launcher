"""Offline Ember production compositor. Run with Pillow 12.3.0; no runtime dependency.
Source: original generated phase-A atlas; exact prompt retained in docs/design/visual-v2/ember-cathedral/prompt.txt.
"""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw, ImageFont
ROOT = Path(__file__).resolve().parents[5]
OUT = Path(__file__).resolve().parent
SOURCE = ROOT / 'docs/design/visual-v2'
a = Image.open(SOURCE/'ember-cathedral/art-atlas.png').convert('RGB')
def save(im, name):
    im.save(OUT/name, optimize=True)
# Mirrored quarters meet with identical edge pixels on both axes: seamless repeat.
q = a.crop((0, 0, 512, 438)).resize((256, 219))
tile = Image.new('RGB', (512, 438))
tile.paste(q, (0,0)); tile.paste(ImageOps.mirror(q), (256,0))
tile.paste(ImageOps.flip(tile.crop((0,0,512,219))), (0,219))
tile = Image.blend(tile, Image.new('RGB', tile.size, '#120C0D'), .38)
save(tile, 'wall-tile.png')
wall = Image.new('RGB', (1280,632))
for y in range(0,632,438):
    for x in range(0,1280,512): wall.paste(tile, (x,y))
save(wall, 'wall.png')
frame = a.crop((512,0,1024,438)).resize((256,220))
# 28 px carved perimeter; center is completely opaque, independent of scaling.
ImageDraw.Draw(frame).rectangle((28,28,227,191), fill='#211719')
save(frame,'frame.png')
for i, name in enumerate(['sigil','sigil-ii','auger']):
    cover=a.crop((i*512,438,(i+1)*512,1024))
    save(cover,name+'.png')
    band=ImageOps.fit(cover,(400,88),centering=(.5,.35))
    save(band,name+'-band.png')
emblem = a.crop((1024,0,1536,438))
save(ImageOps.fit(emblem,(64,64)), 'emblem.png')
# Legible Barlow letterforms; original bronze inset / bone face, no franchise shapes.
logo=Image.new('RGBA',(116,56))
d=ImageDraw.Draw(logo); f=ImageFont.truetype(str(SOURCE/'fonts/barlow-Barlow-Regular.ttf'),46)
d.text((2,-3),'DDL',font=f,fill='#80533B',stroke_width=1)
d.text((1,-5),'DDL',font=f,fill='#F8EAD5')
d.line((2,52,94,52),fill='#B8583F',width=2)
save(logo,'logotype.png')
for p in sorted(OUT.glob('*.png')): print(p.name, p.stat().st_size)
assert sum(p.stat().st_size for p in OUT.glob('*.png')) < 6_000_000
assert list(tile.crop((0,0,1,438)).get_flattened_data()) == list(tile.crop((511,0,512,438)).get_flattened_data())
assert list(tile.crop((0,0,512,1)).get_flattened_data()) == list(tile.crop((0,437,512,438)).get_flattened_data())
