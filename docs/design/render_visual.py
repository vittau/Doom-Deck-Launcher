"""Composite exact UI over original built-in imagegen artwork.
Requires Pillow. Uses the wireframe layout program as the shared information contract.
No image-model text is used. Source art and prompt are recorded in visual-report.md.
"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageOps

ROOT = Path(__file__).resolve().parent
source = (ROOT / 'render_mockups.py').read_text()
# Load layout helpers without executing the screen recipes yet.
exec(source[:source.index("chrome('Home'")], globals())
OUT = ROOT / 'visual'
OUT.mkdir(exist_ok=True)
art = Image.open(ROOT / 'assets/furnace-art-sheet.png').convert('RGB')
arts = [art.crop((i*art.width//3,0,(i+1)*art.width//3,art.height)) for i in range(3)]
C.update(copper='#D87850', edge='#46515B', deep='#090D11')
current = ''
cover_index = 0
original_txt = txt

def box(x,y,w,h,fill='panel',radius=12):
    # Simple Slint-compatible elevation: offset opaque shadow, edge and top highlight.
    if fill in ('panel','raised'):
        d.rounded_rectangle((x,y+4,x+w,y+h+4),radius,fill=C['deep'])
    d.rounded_rectangle((x,y,x+w,y+h),radius,fill=C[fill],outline=C['edge'] if fill=='panel' else None,width=1)
    if fill=='panel':
        d.line((x+16,y+1,x+w-16,y+1),fill=C['edge'],width=1)

def ring(x,y,w,h):
    # Four-pixel canvas gap keeps amber fill and amber focus visually distinct.
    d.rounded_rectangle((x-6,y-6,x+w+6,y+h+6),12,outline=C['bg'],width=4)
    d.rounded_rectangle((x-10,y-10,x+w+10,y+h+10),16,outline=C['amber'],width=4)

def button(x,y,w,label,focused=False,primary=False):
    box(x,y,w,52,'amber' if primary else 'raised',8)
    txt(x+18,y+11,label,22,'bg' if primary else 'text',True)
    # Small directional notch conveys action without requiring a browser effect.
    if w>240:
        col=C['bg'] if primary else C['muted']
        d.line((x+w-27,y+20,x+w-20,y+26,x+w-27,y+32),fill=col,width=2)
    if focused: ring(x,y,w,52)

def row(x,y,w,title,sub=None,focused=False,status=None):
    h=72 if sub else 52
    box(x,y,w,h,'raised',8)
    txt(x+16,y+10,title,22,bold=True)
    if sub: txt(x+16,y+40,sub,20,'muted')
    # Real component switches, in addition to explicit text; no color-only meaning.
    if title in ('Main mapset','Advanced: compat variant Off'):
        on=title=='Main mapset'
        sx=x+w-90; sy=y+12
        d.rounded_rectangle((sx,sy,sx+68,sy+28),14,fill=C['deep'],outline=C['edge'],width=1)
        d.ellipse((sx+42 if on else sx+5,sy+5,sx+60 if on else sx+23,sy+23),fill=C['amber'] if on else C['muted'])
    if focused: ring(x,y,w,h)

def cover(x,y,w,h):
    global cover_index
    idx=cover_index%3 if current=='Discover' else 0
    crop=ImageOps.fit(arts[idx],(int(w),int(h)),method=Image.Resampling.LANCZOS,centering=(.5,.45))
    mask=Image.new('L',crop.size); ImageDraw.Draw(mask).rounded_rectangle((0,0,w-1,h-1),12,fill=255)
    im.paste(crop,(int(x),int(y)),mask)
    d.rounded_rectangle((x,y,x+w,y+h),12,outline=C['edge'],width=1)
    # Exact attribution/disclaimer occupies an opaque plaque, not a textured background.
    d.rectangle((x+8,y+h-34,x+w-8,y+h-8),fill=C['deep'])
    txt(x+14,y+h-32,'Original concept art',18,'muted')
    cover_index+=1

def chrome(active,title,subtitle,hints):
    global im,d,current,cover_index
    current=active; cover_index=0
    im=Image.new('RGB',(1280,800)); d=ImageDraw.Draw(im)
    # A simple linear charcoal gradient with restrained copper structural rails.
    for y in range(800):
        v=y/800; color=tuple(round(a+(b-a)*v) for a,b in zip((18,24,30),(9,13,17)))
        d.line((0,y,1280,y),fill=color)
    d.rectangle((0,0,1280,94),fill=C['deep'])
    d.line((40,94,1240,94),fill=C['edge'],width=1)
    d.line((40,95,144,95),fill=C['copper'],width=2)
    # DDL split-gate logotype: original vector-friendly polygons, exact Inter letters.
    d.polygon([(40,28),(53,20),(53,28),(47,32),(47,60),(53,64),(53,72),(40,64)],fill=C['copper'])
    d.polygon([(76,28),(63,20),(63,28),(69,32),(69,60),(63,64),(63,72),(76,64)],fill=C['copper'])
    d.rectangle((57,27,60,65),fill=C['amber'])
    txt(88,26,'DDL',32,'text',True)
    for x,label in [(232,'Home'),(392,'Discover'),(572,'Library'),(752,'Settings')]:
        if label==active:
            box(x-18,20,140,52,'raised',8)
            d.line((x,76,x+100,76),fill=C['amber'],width=3)
        txt(x,34,label,22,'text' if label==active else 'muted',label==active)
    # Product-name clarification replaces the confusing Native / DSDA string.
    txt(1020,37,'DoomDeckLauncher',20,'muted')
    txt(40,125,title,36,bold=True)
    if subtitle: txt(40,175,subtitle,22,'muted')
    d.line((40,211,1240,211),fill=C['edge'],width=1)
    d.rectangle((0,736,1280,800),fill=C['deep'])
    d.line((40,736,1240,736),fill=C['edge'],width=1)
    x=40
    for key,verb in hints:
        width=max(34,int(d.textlength(key,font=font(18,True)))+16)
        if key in ('A','B','X','Y'):
            d.ellipse((x,754,x+32,786),fill=C['raised'],outline=C['edge'])
            txt(x+9,758,key,18,'muted' if key=='B' and title in ('Pick up where you left off','Discover') else 'amber',True)
        else:
            box(x,754,width,30,'raised',6); txt(x+8,758,key,18,'amber',True)
        txt(x+width+8,756,verb,20,'muted'); x+=width+20+d.textlength(verb,font=font(20))+22

keep={'home','discover','mapset-detail','play-setup-sigil','play-blocked-iwad','download-install'}
def save(name):
    if name not in keep: return
    # The option-dense setup screens get a compact original-art identifier in unused title space.
    if name.startswith('play-'):
        crop=ImageOps.fit(arts[0],(220,80),method=Image.Resampling.LANCZOS)
        im.paste(crop,(1020,120))
        d.rectangle((1020,174,1240,200),fill=C['deep'])
        txt(1030,177,'Original concept art',18,'muted')
    im.save(OUT/(name+'.png'),optimize=True)

# Shared screen recipes retain every exact string, prerequisite, component and button legend.
recipes=source[source.index("chrome('Home'"):source.index('# Deterministic artifact checks')]
exec(recipes, globals())
for p in sorted(OUT.glob('*.png')):
    with Image.open(p) as pic: assert pic.size==(1280,800),p
    assert p.stat().st_size<2_000_000,p
    print(f'{p.name}: 1280 x 800, {p.stat().st_size:,} bytes')
