"""Original DDL flat UI mockups. Requires Pillow; run from any directory.
Font: unmodified Inter 4.1, SIL OFL 1.1, assets/Inter-LICENSE.txt.
No artwork, screenshots, browser effects or application code are used.
"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent
OUT = ROOT / 'mockups'
C = dict(bg='#101418', panel='#1B2229', raised='#26313B', text='#F4F1EA',
         muted='#B9C3CB', amber='#FFCA70', green='#88D9B0', red='#FFA69C', line='#667482')
fonts = {}
def font(size=22, bold=False):
    key = (size, bold)
    if key not in fonts:
        f = ImageFont.truetype(str(ROOT / 'assets/InterVariable.ttf'), size)
        f.set_variation_by_axes([14, 650 if bold else 400])
        fonts[key] = f
    return fonts[key]
def txt(x,y,s,size=22,color='text',bold=False):
    d.text((x,y),s,font=font(size,bold),fill=C.get(color,color))
def box(x,y,w,h,fill='panel',radius=12):
    d.rounded_rectangle((x,y,x+w,y+h),radius,fill=C[fill])
def ring(x,y,w,h):
    d.rounded_rectangle((x-8,y-8,x+w+8,y+h+8),16,outline=C['amber'],width=4)
def button(x,y,w,label,focused=False,primary=False):
    box(x,y,w,52,'amber' if primary else 'raised',8)
    txt(x+18,y+11,label,22,'bg' if primary else 'text',True)
    if focused: ring(x,y,w,52)
def row(x,y,w,title,sub=None,focused=False,status=None):
    h=72 if sub else 52
    box(x,y,w,h,'raised',8)
    txt(x+16,y+10,title,22,bold=True)
    if sub: txt(x+16,y+40,sub,20,'muted')
    if status: txt(x+w-90,y+12,status,20,'amber')
    if focused: ring(x,y,w,h)
def cover(x,y,w,h):
    box(x,y,w,h,'raised')
    # Deliberately abstract geometry, not a map screenshot or franchise artwork.
    d.line((x+24,y+h-35,x+w-24,y+35),fill=C['line'],width=2)
    d.line((x+24,y+35,x+w-24,y+h-35),fill=C['line'],width=2)
    label='cover art'
    tw=d.textlength(label,font=font(22))
    box(x+w/2-tw/2-16,y+h/2-23,tw+32,48)
    txt(x+w/2-tw/2,y+h/2-14,label,22,'muted')
def chrome(active,title,subtitle,hints):
    global im,d
    im=Image.new('RGB',(1280,800),C['bg']); d=ImageDraw.Draw(im)
    txt(40,26,'DDL',30,'amber',True)
    for x,label in [(180,'Home'),(340,'Discover'),(520,'Library'),(680,'Settings')]:
        txt(x,30,label,22,'text' if label==active else 'muted',label==active)
        if label==active: d.rectangle((x,70,x+100,73),fill=C['amber'])
    d.line((40,103,1240,103),fill=C['line'])
    txt(40,125,title,36,bold=True)
    if subtitle: txt(40,175,subtitle,22,'muted')
    d.rectangle((0,736,1280,800),fill=C['panel'])
    x=40
    for key,verb in hints:
        width=max(34,int(d.textlength(key,font=font(18,True)))+16)
        box(x,754,width,30,'raised',6); txt(x+8,758,key,18,'muted' if key=='B' and title in ('Pick up where you left off','Discover','Your library','Settings','Welcome to DoomDeckLauncher') else 'amber',True)
        txt(x+width+8,756,verb,20,'muted'); x+=width+20+d.textlength(verb,font=font(20))+22

def save(name): im.save(OUT/(name+'.png'),optimize=True)

chrome('Home','Pick up where you left off','Your last profile, ready again.', [('A','Continue'),('B','Back'),('X','Details'),('L1/R1','Tabs'),('Start','Menu')])
box(40,230,784,298); cover(64,254,224,250)
txt(320,255,'CONTINUE',20,'amber',True); txt(320,292,'SIGIL',44,bold=True)
txt(320,352,'John Romero',22,'muted'); txt(320,394,'Ultimate DOOM · Default profile',22)
button(320,450,220,'Continue',True,True); button(560,450,220,'Details')
box(848,230,392,298); txt(872,254,'Ready to play',26,bold=True)
txt(872,308,'✓ Engine installed',22,'green'); txt(872,350,'✓ Ultimate DOOM found',22,'green')
txt(872,392,'Verification pending',20,'amber'); button(872,450,344,'Discover mapsets')
txt(40,558,'Recently played',26,bold=True)
row(40,606,584,'SIGIL · John Romero','Ultimate DOOM · Last played today')
row(656,606,584,'AUGER;ZENITH · DBP 37','DOOM II · Last played yesterday'); save('home')

chrome('Discover','Discover','Curated mapsets. Clear requirements before you install.', [('A','Open'),('B','Back'),('X','Favorite'),('L1/R1','Tabs'),('Start','Menu')])
for x,w,label in [(40,152,'Featured'),(208,224,'Beginner Friendly'),(448,140,'Classics'),(604,164,'Cacowards'),(784,456,'Doomer Boards Projects')]:
    box(x,230,w,52,'raised' if x==40 else 'panel',8); txt(x+14,244,label,20,'amber' if x==40 else 'text',x==40)
for x,title,author,game,ready in [(40,'SIGIL','John Romero','Ultimate DOOM','Base game ready'),(448,'AUGER;ZENITH','Doomer Boards Project 37','DOOM II','Base game ready'),(856,'SIGIL II','John Romero','Ultimate DOOM','Base game ready')]:
    box(x,314,384,386); cover(x+16,330,352,154)
    txt(x+20,502,title,26,bold=True); txt(x+20,544,author,20,'muted'); txt(x+20,581,game,22)
    txt(x+20,621,ready,20,'green'); txt(x+20,659,'Verification pending',20,'amber')
ring(40,314,384,386); save('discover')

chrome('Discover','Mapset detail','Curated recipe / SIGIL 1.23', [('A','Play setup'),('B','Back'),('X','Favorite'),('Start','Menu')])
cover(40,230,360,330); button(40,592,360,'Play setup',True,True); button(40,660,360,'Profiles · 2')
box(432,230,808,482); txt(456,254,'SIGIL',44,bold=True); txt(456,314,'By John Romero',22,'muted')
txt(456,365,'A new episode for the original DOOM.',26); txt(456,405,'Free main mapset with included MIDI music.',22,'muted')
row(456,455,760,'Ultimate DOOM','Matching base game found')
row(456,543,760,'Verification pending','Complevel recommendation: 3 · needs Deck validation')
txt(456,638,'Optional soundtrack: user-supplied',22); txt(456,677,'Source & credits: romero.com/sigil',20,'muted'); save('mapset-detail')

def setup(blocked=False,auger=False):
    title='AUGER;ZENITH' if auger else 'SIGIL'
    chrome('Library','Play setup / '+title,'Changes are saved to this mapset’s profile.', [('A','Inspect' if auger else 'Choose folder' if blocked else 'Play'),('B','Back'),('X','Profiles'),('Y','Launch details'),('Start','Menu')])
    box(40,230,680,482); box(744,230,496,482)
    row(64,254,632,'Profile: Default','Independent options and saves')
    row(64,342,632,'Main mapset','Required · enabled')
    if auger:
        row(64,430,632,'Optional add-on','Enabled · loads after the main PWAD')
        row(64,518,632,'Skill 3','Player preference')
        row(64,606,632,'Pistol start: Off','Reset inventory at each map')
    else:
        row(64,430,632,'Soundtrack: not provided — import','Optional · included MIDI music is available')
        row(64,518,632,'Advanced: compat variant Off','Mutually exclusive with main mapset')
        row(64,606,304,'Skill 3'); row(384,606,312,'Pistol start: Off')
        txt(64,672,'Load order follows the selected recipe.',20,'muted')
    txt(768,254,'Will launch',26,bold=True)
    game='DOOM II' if auger else 'Ultimate DOOM'
    txt(768,298,game,26,'amber' if blocked else 'text',True)
    txt(768,337,'Missing base game' if blocked else '✓ Matching IWAD found',22,'red' if blocked else 'green')
    txt(768,380,'2 content files · order 1 → 2' if auger else '1 content file · load order 1',22)
    txt(768,424,'1  Main mapset' if auger else '1  SIGIL_V1_23.wad',22)
    txt(768,462,'2  Optional add-on' if auger else 'DSDA-Doom v0.30.1',22,'muted')
    txt(768,504,'Complevel: awaiting evidence' if auger else 'Complevel 3 · not yet verified',20,'amber')
    if blocked:
        txt(768,548,'Add your legally owned IWAD.',22)
        button(768,592,448,'Add IWAD folder',True,True)
        txt(768,670,'Play unavailable · IWAD missing',20,'red')
    elif auger:
        button(768,592,448,'View recipe evidence',True)
        txt(768,670,'Play unavailable · recipe incomplete',20,'red')
    else:
        txt(768,548,'Skill 3 · Pistol start Off',22,'muted')
        button(768,610,448,'Play',True,True)
        txt(768,678,'Soundtrack is optional.',20,'muted')
setup(); save('play-setup-sigil')
setup(True); save('play-blocked-iwad')
setup(auger=True); save('play-setup-auger')

chrome('Discover','Installing SIGIL','Keep browsing while the installation runs.', [('A','Cancel'),('B','Back'),('X','Details'),('Start','Menu')])
cover(40,230,288,330); box(360,230,880,482)
txt(392,262,'Downloading main package',30,bold=True); txt(392,315,'Official source · romero.com',22,'muted')
for x,label,col in [(392,'1 Download','amber'),(610,'2 Verify','muted'),(798,'3 Extract','muted'),(1000,'4 Install','muted')]: txt(x,382,label,22,col)
box(392,438,816,20,'raised',8); box(392,438,506,20,'amber',8)
txt(392,487,'3.1 MB / 5.0 MB',26,bold=True); txt(905,490,'62% · example',22,'muted')
txt(392,536,'Files are checked before they enter your library.',22,'muted')
button(392,614,260,'Cancel',True); button(676,614,260,'Details')
txt(40,600,'SIGIL 1.23',26,bold=True); txt(40,643,'Illustrative progress data',20,'muted'); save('download-install')

chrome('Settings','Settings','Manage your engine, game folders and local data.', [('A','Check updates'),('B','Back'),('L1/R1','Tabs'),('Start','Menu')])
box(40,230,312,482)
for i,s in enumerate(['Engine','IWAD folders','Storage','DSDA config','Diagnostics','Appearance']):
    y=254+i*72
    if i==0: box(56,y,280,52,'raised',8)
    txt(72,y+12,s,22,'amber' if i==0 else 'text',i==0)
box(376,230,864,482); txt(400,254,'DSDA-Doom',30,bold=True)
txt(400,310,'Installed: v0.30.1 · research reference',22); txt(400,350,'Release trust and gameplay checks are separate.',22,'muted')
button(400,410,380,'Check for updates',True,True)
row(400,492,816,'Roll back','Choose a previous installed version')
row(400,580,816,'Advanced: unlisted release','Not validated · explicit warning before installation')
txt(400,675,'Updates never interrupt a running game.',20,'muted'); save('settings')

chrome('','Welcome to DoomDeckLauncher','Get ready in Gaming Mode, with your controller.', [('A','Install engine'),('B','Back'),('Start','Menu')])
box(40,230,584,482); box(648,230,592,482)
txt(64,260,'1  Install the engine',30,bold=True); txt(64,320,'DSDA-Doom runs your mapsets.',22)
txt(64,365,'DDL downloads the official engine',22,'muted'); txt(64,399,'and checks it before installation.',22,'muted')
txt(64,458,'Game data is not included.',22,'amber'); button(64,530,536,'Install DSDA-Doom',True,True)
txt(64,625,'No shell commands or keyboard needed.',22,'muted')
txt(672,260,'2  Add your base games',30,bold=True); txt(672,320,'Use IWADs you legally own.',22)
txt(672,365,'Choose a folder, review the detected',22,'muted'); txt(672,399,'games, then confirm your selection.',22,'muted')
button(672,530,544,'Add IWAD folder'); button(672,610,544,'Browse without a base game')
txt(672,679,'Play stays blocked until setup is ready.',20,'muted'); save('first-run')

chrome('Library','Your library','Installed mapsets, local imports and saved profiles.', [('A','Open'),('B','Back'),('X','Favorite'),('Y','Import'),('L1/R1','Tabs'),('Start','Menu')])
for x,w,s in [(40,160,'All'),(216,160,'Ready'),(392,220,'Needs setup'),(628,200,'Favorites')]:
    box(x,230,w,52,'raised' if x==40 else 'panel',8); txt(x+20,244,s,20,'amber' if x==40 else 'text')
for x,title,creator,game,last in [(40,'SIGIL','John Romero','Ultimate DOOM','Today'),(656,'AUGER;ZENITH','Doomer Boards Project 37','DOOM II','Yesterday')]:
    box(x,314,584,386); cover(x+24,338,156,200); txt(x+204,346,title,26,bold=True)
    txt(x+204,394,creator,20,'muted'); txt(x+204,436,game,22)
    txt(x+24,570,'Verification pending · 2 profiles',22,'amber')
    txt(x+24,614,'Last played: '+last,22,'muted'); button(x+24,654,536,'Open mapset')
ring(40,314,584,386); save('library')

# Deterministic artifact checks: dimensions, byte ceiling, one ring per screen
# (exactly one unique amber outline is constructed in each screen above).
for p in sorted(OUT.glob('*.png')):
    with Image.open(p) as pic: assert pic.size==(1280,800), p
    assert p.stat().st_size < 1_500_000, p
    print(f'{p.name}: 1280 x 800, {p.stat().st_size:,} bytes')
