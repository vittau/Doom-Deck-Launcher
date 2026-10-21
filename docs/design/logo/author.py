"""Author the production logo masters (D-029, spec 039): Rose Compass · Stained Glass.

One authoring source: the geometry, palette and outlined lettering come from
../logo-v3/author.py (imported, never copied). This file only composes the
production masters and the Steam compositions from it (their scenery comes
from scene.py), then compiles the masters into the app. Design time only: fonttools==4.60.1. Exporting the
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
_spec = importlib.util.spec_from_file_location('steam_scene', HERE / 'scene.py')
steam = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(steam)
num, path, group, svg, lettering = v3.num, v3.path, v3.group, v3.svg, v3.lettering
DARK, BONE = v3.DARK, v3.BONE


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

# README variants retain the lock-up and padding, with no background tablet.
def readme_logo(ink=BONE):
    readme_name, _ = lettering('DoomDeckLauncher', 38, ink)
    content = group(mark, 0, 0, .625) + group(readme_name, 184, 54, 1.45)
    return group(content, 30, 20)
for theme, ink in [('dark', BONE), ('light', DARK)]:
    save(f'readme-logo-{theme}.svg', 760, 200, readme_logo(ink), full[0])

# README compact header (spec 081): the website header's small-tier emblem, DDL
# wordmark and Barlow subtitle, composed from the same masters. Geometry is the
# header lock-up (emblem at 128, DDL at 152,30); the theme only swaps fills.
README_HEADER = {'dark': (BONE, '#A8302A', BONE), 'light': (DARK, '#A8302A', DARK)}
def readme_header(theme):
    face, depth, ink = README_HEADER[theme]
    subtitle, _ = lettering('DoomDeckLauncher', 22, ink)
    content = group(small[1], 0, 0, .5) + group(v3.wordmark(face, depth), 152, 30) + group(subtitle, 152, 112)
    return group(content, 12, 12)
for theme in README_HEADER:
    save(f'readme-header-{theme}.svg', 370, 160, readme_header(theme), lock[0])

# Steam compositions (spec 054): the lettering sits directly on the Ember Cathedral
# scenes from scene.py, which shade the areas it uses; the hero has no mark or text.
lib, wide, hero = steam.library_capsule(), steam.wide_capsule(), steam.hero()
steam_art = {'library-capsule': lib[1] + group(mark, 172, 100) + group(wordmark, 300 - WM_C * 2, 398, 2) + group(name, (600 - nw) / 2, 620),
             'wide-capsule': wide[1] + group(mark, 80, 90, .9) + group(wordmark, 336, 130, 1.3) + group(name, 336, 268, .8),
             'hero': hero[1]}
save('library-capsule.svg', 600, 900, steam_art['library-capsule'], full[0] + lib[0])
save('wide-capsule.svg', 920, 430, steam_art['wide-capsule'], full[0] + wide[0])
save('hero.svg', 3840, 1240, steam_art['hero'], hero[0])

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
        if row == 0:
            b += v3.label('stacked 240×200', 900, y0 + 24, 18, ink)
            b += group(stacked, 880, y0 + 60, .5)
        b += v3.label(f'README header {"dark" if row == 0 else "light"} theme 370×160 (shown at 1/2)', 40, y0 + 462, 18, ink)
        b += group(readme_header('dark' if row == 0 else 'light'), 40, y0 + 490, .5)
    # Steam art, each in a nested viewport that clips its scene to the frame.
    def framed(composition, w, h, x, y, scale):
        return group(f'<svg width="{num(w * scale)}" height="{num(h * scale)}" viewBox="0 0 {w} {h}">{composition}</svg>', x, y)
    b += path('M0 1200H1200V2020H0Z', '#1A1213')
    b += v3.label('Library capsule 600×900, 1/3', 40, 1224, 18, v3.MUTED)
    b += framed(steam_art['library-capsule'], 600, 900, 40, 1260, 1 / 3)
    b += v3.label('Wide capsule 920×430, 1/2', 300, 1224, 18, v3.MUTED)
    b += framed(steam_art['wide-capsule'], 920, 430, 300, 1260, .5)
    b += v3.label('Hero 3840×1240, 7/24 (no mark or text; Steam draws logo.png over it)', 40, 1584, 18, v3.MUTED)
    b += framed(steam_art['hero'], 3840, 1240, 40, 1620, 7 / 24)
    return b
save('contact-sheet.svg', 1200, 2020, sheet(), full[0] + lock[0] + lib[0] + wide[0] + hero[0])

# Exact same masters compiled into Slint, with byte equality verified by export.
for stem in ['emblem', 'emblem-small', 'horizontal', 'stacked', 'full-name']:
    (ROOT / f'crates/ddl/ui/assets/logo/{stem}.svg').write_bytes((HERE / f'{stem}.svg').read_bytes())

save('steam-logo.svg', 1280, 720, group(mark, 465, 30, 1.37) + group(wordmark, 640 - WM_C * 2.5, 404, 2.5) + group(name, (1280 - nw * 2) / 2, 618, 2), full[0])
# Social scenes have their own original raster layers and outlined copy.
# Rebuild after the production masters above, never restore the old scenery.
runpy.run_path(str(HERE / 'author-social.py'), run_name='__main__')
