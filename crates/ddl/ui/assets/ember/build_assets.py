"""Offline Ember production compositor. Run with Pillow 12.3.0; no runtime dependency.
Source: original generated phase-A atlas; exact prompt retained in docs/design/visual-v2/ember-cathedral/prompt.txt.
Painted Eviternity/fallback PNG masters are preserved; prompts retained below.
"""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw, ImageFont
# 2026-10-16 cover repaint: built-in image_gen, style references only:
# sigil.png and sigil-ii.png. Original concept art under repository GPL-3.0-or-later.
# Generated covers are committed raster masters; offline regeneration keeps them
# and deterministically derives their bands. Re-generation is not byte reproducible.
COVER_PROMPTS = {
    'fallback': """Use case: stylized-concept
Asset type: original generic community-mapset cover, one portrait painting at 512:586 aspect ratio, larger resolution allowed.
Primary request: a single continuous painted landscape of a weathered carved gate standing in a distant rocky hellscape. View the complete small arched gateway in three-quarter perspective on a diagonal crumbling causeway, surrounded by rolling smoke, rugged basalt slopes and restrained red-orange firelight. The gate is a mid-distance focal point integrated into the terrain, with its whole arch visible and only short tapered supports; no tall foreground pillars. Generic original environment with no recognizable mapset symbols.
Input image: SIGIL cover is a reference only for rich painterly detail, palette, warm ember illumination and atmospheric depth, not architecture or composition.
Style/medium: richly textured atmospheric digital matte painting, painterly smoke and clouds flowing continuously across the full width, fine stone texture, natural irregular silhouettes, dramatic chiaroscuro. Deep oxblood/brown-black shadows, aged bronze carving, warm orange firelight and small bone highlights.
Composition: full-bleed portrait single coherent scene with one perspective and continuous terrain. Smoky rolling sky in upper third; small complete arch near the center with surrounding slopes; diagonal causeway and glowing fissures lead through lower foreground. The upper-middle horizontal crop must read as a connected landscape, not vertical strips. Architecture must not form edge-to-edge vertical dividing lines.
Constraints: entirely original concept art. No panels, triptych, collage, tiling, side-by-side images, seams, giant vertical slabs, symmetrical pillar walls, borders, text, letters, logos, UI, plaques, franchise material, screenshots, pentagrams, skulls or flat vector shapes. UI adds its own Original concept art plaque.""",
    'fallback-initial-rejected': """Use case: stylized-concept
Asset type: generic original painted community-mapset cover for DDL Ember Cathedral UI, portrait 512:586 aspect ratio.
Primary request: a massive weathered carved stone gate opening onto a distant atmospheric hellscape. Broad dark basalt threshold, aged bronze inset rails and subtle abstract chisel patterns, firelight spilling across worn steps, red-orange glowing fissures and smoky distant ruined silhouettes beyond. Tasteful and generic, suitable for any community mapset; no distinctive mapset symbol or recognizable specific location.
Input images: SIGIL and SIGIL II paintings are references for painted detail, palette and light only; do not copy their layouts.
Style/medium: richly detailed atmospheric digital matte painting, fine rough stone textures, painterly smoke, cinematic depth and natural irregular edges. Match reference detail and restraint. Deep oxblood/brown-black shadow, scorched bronze, warm orange ember highlights with small bone accents.
Composition/framing: portrait full-bleed single scene, gate fills the upper-middle with grounded foreground steps and cavernous layered depth. Keep the gateway readable in both the full portrait and a wide horizontal crop across the upper-middle 35 percent. No giant city, no eclipse, no mascot.
Constraints: entirely original art; no text, no letters, no logos, no plaque baked in (UI adds Original concept art plaque), no id Software material, no franchise imagery, no game screenshots, no pentagrams, no skulls, no flat vector shapes, no border.""",
    'eviternity': """Use case: stylized-concept
Asset type: original painted mapset cover for DDL Ember Cathedral UI, portrait 512:586 aspect ratio.
Primary request: Paint an eternal celestial-yet-infernal citadel, ivory and aged gold against hellfire. An immense weathered ivory cathedral-city rises out of dark basalt chasms and fiery red-orange mist, elegant gold ribs and distant spires disappearing into turbulent charcoal clouds, a pale celestial glow behind the central crown, infernal molten light far below. Original imagined architecture, not any existing game's location or title art.
Input images: attached SIGIL and SIGIL II paintings are style/palette/detail references only, never copy their layouts.
Style/medium: richly detailed atmospheric digital matte painting, painterly stone and clouds, layered depth, natural irregular edges, muted weathered materials, dramatic chiaroscuro. Match the reference paintings' fine architectural detail and orange ember illumination, deep oxblood/brown-black shadows, restrained bone highlights.
Composition/framing: portrait full-bleed single scene, imposing central citadel in middle distance, bridge and foreground broken stone for depth. Keep the primary architecture readable in both the full portrait and a wide horizontal crop across the upper-middle 35 percent.
Constraints: original concept art only; no text, no letters, no logos, no plaque baked in (the UI adds its Original concept art plaque), no recognizable franchise imagery, no game screenshots, no vector shapes, no flat geometric illustration, no borders.""",
}
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
for i, name in enumerate(['sigil','sigil-ii','auger','eviternity','fallback']):
    if i < 3:
        cover=a.crop((i*512,438,(i+1)*512,1024))
        save(cover,name+'.png')
    else:
        # Committed painted masters: never replace these with SVG exports.
        cover=Image.open(OUT/(name+'.png')).convert('RGB')
        assert cover.size == (512,586), name
        assert (OUT/(name+'.png')).stat().st_size <= 600_000, name
    # One continuous source crop, with the transition into the title baked in.
    band=ImageOps.fit(cover,(400,88),centering=(.5,.35))
    fade=Image.new('L',band.size)
    for x in range(400):
        t=min(1., x/144.)
        alpha=round(255 * t*t*(3-2*t))
        ImageDraw.Draw(fade).line((x,0,x,87),fill=alpha)
    band=Image.composite(band,Image.new('RGB',band.size,'#120C0D'),fade)
    save(band,name+'-band.png')
    assert (OUT/(name+'-band.png')).stat().st_size <= 600_000, name
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
