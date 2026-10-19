"""Author the production logo masters (D-029, spec 039): Rose Compass · Stained Glass.

One authoring source: the geometry, palette and outlined lettering come from
../logo-v3/author.py (imported, never copied). This file only composes the
production masters and the Steam compositions from it, then compiles the
masters into the app. Design time only: fonttools==4.60.1. Exporting the
committed SVGs needs only resvg (see export.py).
"""
import importlib.util
import runpy
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
_spec = importlib.util.spec_from_file_location('logo_v3_author', HERE.parent / 'logo-v3/author.py')
v3 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(v3)
path, group, svg, lettering = v3.path, v3.group, v3.svg, v3.lettering
DARK, BONE, BRONZE, EMBER = v3.DARK, v3.BONE, v3.BRONZE, v3.EMBER


def save(name, w, h, body, defs=''):
    (HERE / name).write_text(svg(w, h, body, defs))


# Stained Glass (owner pick, D-029): full tier from 128 px, small tier for 32-64 px.
spec = v3.VARIANTS['stained-glass']
full, small = spec['full'](), spec['small']()
mark = full[1]
wordmark = spec['ddl']()[1]
lock = v3.lockup(small, spec['ddl']())
name, nw = lettering('DoomDeckLauncher', 38, BONE)
# The v3 wordmark (solid letters with an outline and a carved side) spans
# x -3.5..160.5 around the D-023 contours; WM_C is its centre.
WM_C = 78.5
save('emblem.svg', 256, 256, mark, full[0])
save('emblem-small.svg', 256, 256, small[1], small[0])
save('horizontal.svg', 330, 128, lock[1], lock[0])
save('stacked.svg', 480, 400, group(mark, 112, 0) + group(wordmark, 240 - WM_C, 269) + group(name, (480 - nw) / 2, 354), full[0])
save('full-name.svg', 700, 160, group(mark, 0, 0, .625) + group(name, 184, 54, 1.45), full[0])

# Original geometric woodcut scenery: repeated carved buttresses, stars, chasm,
# stairs and tiny arch windows. Eviternity is mood art, never a map screenshot.
def scene(ivory=False):
    s=path('M0 0H1200V900H0Z',DARK)
    s+=path('M0 0H1200V650L600 810 0 650Z','#27181F' if ivory else '#30181C')
    s+=path('M600 70 910 370 600 680 290 370Z','#6B4541' if ivory else '#61312B')
    s+=path('M600 104 875 370 600 646 325 370Z','#B7A58A' if ivory else '#89563B')
    s+=path('M600 135 845 370 600 615 355 370Z','#30292E' if ivory else '#30181C')
    for i in range(68):
        x=(i*173+37)%1200; y=(i*79+23)%540
        s+=path(f'M{x} {y-2} {x+2} {y} {x} {y+2} {x-2} {y}Z','#B88960')
    for side in (False,True):
        buildings=''
        for i in range(8):
            x=i*66-40; top=420-i*35; col='#554342' if ivory else '#412629'
            buildings+=path(f'M{x} 900V{top}L{x+32} {top-75} {x+64} {top}V900Z',col)
            buildings+=path(f'M{x+8} 900V{top+8}L{x+32} {top-43} {x+32} 900Z','#8C7965' if ivory else '#714733')
            for y in range(top+42,850,88):
                buildings+=path(f'M{x+20} {y+36}V{y}L{x+32} {y-17} {x+44} {y}V{y+36}Z',DARK)
                buildings+=path(f'M{x+29} {y+29}V{y}L{x+32} {y-5} {x+35} {y}V{y+29}Z', '#CDBA91' if ivory else EMBER)
        s+=f'<g transform="translate({1200 if side else 0} 0) scale({-1 if side else 1} 1)">{buildings}</g>'
    s+=path('M487 900V445L600 290 713 445V900Z','#947A60' if ivory else '#80533B')
    s+=path('M508 900V454L600 328 692 454V900Z',DARK)
    s+=path('M536 900V467L600 376 664 467V900Z','#3C252B')
    s+=path('M600 430 616 570 638 603 600 664 562 603 584 570Z','#FFD078' if ivory else EMBER)
    for i in range(9):
        y=690+i*26; half=70+i*26
        s+=path(f'M{600-half} {y}H{600+half}V{y+9}H{600-half}Z','#B7A58A' if ivory else '#80533B')
    s+=path('M0 875 240 834 414 900H0Z M1200 875 960 834 786 900H1200Z',DARK)
    return s


# README logo: the full name on a dark tablet (octagon-style corner cuts), so the bone
# lettering stays readable on GitHub's light and dark themes alike.
def readme_logo():
    content = group(mark, 0, 0, .625) + group(name, 184, 54, 1.45)
    tablet = path(v3.poly([(14, 0), (746, 0), (760, 14), (760, 186), (746, 200), (14, 200), (0, 186), (0, 14)]), DARK)
    return tablet + group(content, 30, 20)
save('readme-logo.svg', 760, 200, readme_logo(), full[0])

# Contact sheet of the production set, itself vector (labels are outlined Barlow).
def sheet():
    stacked = group(mark, 112, 0) + group(wordmark, 240 - WM_C, 269) + group(name, (480 - nw) / 2, 354)
    b = ''
    for row, (bg, ink) in enumerate([(DARK, v3.MUTED), (BONE, '#4A3A33')]):
        y0 = row * 600
        b += path(f'M0 {y0}H1200V{y0 + 600}H0Z', bg)
        b += v3.label('small tier (window icon, header): 32, 48, 64 px', 40, y0 + 24, 18, ink)
        x = 40
        for size in (32, 48, 64):
            b += group(small[1], x, y0 + 128 - size, size / 256)
            b += v3.label(str(size), x + size / 2, y0 + 140, 16, ink, centre=True)
            x += size + 40
        b += v3.label('full tier: 128 and 256 px', 440, y0 + 24, 18, ink)
        for size, fx in ((128, 440), (256, 600)):
            b += group(mark, fx, y0 + 320 - size, size / 256)
            b += v3.label(f'{size} px', fx + size / 2, y0 + 332, 16, ink, centre=True)
        b += v3.label('horizontal lock-up, native 330×128', 40, y0 + 180, 18, ink)
        b += group(lock[1], 40, y0 + 208)
        b += v3.label('header size 154×56', 40, y0 + 356, 18, ink)
        b += group(lock[1], 40, y0 + 386, 56 / 128)
        b += v3.label('stacked 240×200' if row == 0 else 'README logo 380×100', 900 if row == 0 else 40, y0 + 24 if row == 0 else y0 + 462, 18, ink)
        if row == 0:
            b += group(stacked, 880, y0 + 60, .5)
            b += v3.label('full name 350×80', 40, y0 + 462, 18, ink)
            b += group(group(mark, 0, 0, .625) + group(name, 184, 54, 1.45), 40, y0 + 490, .5)
        else:
            b += group(readme_logo(), 40, y0 + 490, .5)
    return b
save('contact-sheet.svg', 1200, 1200, sheet(), full[0] + lock[0])

# Exact same masters compiled into Slint, with byte equality verified by export.
for stem in ['emblem', 'emblem-small', 'horizontal', 'stacked', 'full-name']:
    (ROOT / f'crates/ddl/ui/assets/logo/{stem}.svg').write_bytes((HERE / f'{stem}.svg').read_bytes())

# Dedicated Steam compositions retain an open central hero safe area.
art = scene()
save('library-capsule.svg', 600, 900, group(art, -300, 0) + path('M90 370H510V680H90Z', DARK) + group(mark, 172, 100) + group(wordmark, 300 - WM_C * 2, 398, 2) + group(name, (600 - nw) / 2, 620), full[0])
save('wide-capsule.svg', 920, 430, group(art, 0, -240, .767) + path('M40 50H650V370H40Z', DARK) + group(mark, 64, 90, .9) + group(wordmark, 320, 130, 1.3) + group(name, 320, 268, .8), full[0])
save('hero.svg', 3840, 1240, group(art, 0, -800, 3.2))
save('steam-logo.svg', 1280, 720, group(mark, 465, 30, 1.37) + group(wordmark, 640 - WM_C * 2.5, 404, 2.5) + group(name, (1280 - nw * 2) / 2, 618, 2), full[0])
# Social scenes have their own original raster layers and outlined copy.
# Rebuild after the production masters above, never restore the old scenery.
runpy.run_path(str(HERE / 'author-social.py'), run_name='__main__')
