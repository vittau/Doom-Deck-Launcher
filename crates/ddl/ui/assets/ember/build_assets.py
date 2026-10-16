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
source_frame = a.crop((512,0,1024,438))
f = source_frame.resize((256,220), Image.Resampling.LANCZOS)
# Only plain rail cross-sections stretch. Corners and ornaments never scale up.
frame = Image.new('RGB', (128,128), '#211719')
for x,y,sx,sy in [(0,0,0,0),(100,0,228,0),(0,100,0,192),(100,100,228,192)]:
    frame.paste(f.crop((sx,sy,sx+28,sy+28)), (x,y))
frame.paste(f.crop((60,0,61,28)).resize((72,28)), (28,0))
frame.paste(f.crop((60,192,61,220)).resize((72,28)), (28,100))
frame.paste(f.crop((0,60,28,61)).resize((28,72)), (0,28))
frame.paste(f.crop((228,60,256,61)).resize((28,72)), (100,28))
save(frame, 'frame.png')
for name, box, size in [
    ('ornament-top.png', (208,0,304,56), (96,28)),
    ('ornament-bottom.png', (208,382,304,438), (96,28)),
    ('ornament-left.png', (0,190,56,248), (28,48)),
    ('ornament-right.png', (456,190,512,248), (28,48)),
]:
    save(source_frame.crop(box).resize(size, Image.Resampling.LANCZOS), name)
for i, name in enumerate(['sigil','sigil-ii','auger']):
    cover=a.crop((i*512,438,(i+1)*512,1024))
    save(cover,name+'.png')
    # One continuous source crop, with the transition into the title baked in.
    band=ImageOps.fit(cover,(400,88),centering=(.5,.35))
    fade=Image.new('L',band.size)
    for x in range(400):
        t=min(1., x/144.)
        alpha=round(255 * t*t*(3-2*t))
        ImageDraw.Draw(fade).line((x,0,x,87),fill=alpha)
    band=Image.composite(band,Image.new('RGB',band.size,'#120C0D'),fade)
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
