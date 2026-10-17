"""Author social compositions from production paths and outlined Barlow text.

Design-time only: fonttools==4.60.1. Exporting these SVGs does not need fonttools.
Raster scenes are original built-in image_gen outputs; see layers/prompts.json.
"""
from copy import deepcopy
from pathlib import Path
import xml.etree.ElementTree as ET

from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)
font = TTFont(ROOT / 'crates/ddl/ui/assets/Barlow-Regular.ttf')
glyphs = font.getGlyphSet()
cmap = font.getBestCmap()


def node(tag, **attrs):
    return ET.Element(f'{{{NS}}}{tag}', {k: str(v) for k, v in attrs.items()})


def placed(children, x, y, scale=1, **attrs):
    group = node('g', transform=f'translate({x} {y}) scale({scale})', **attrs)
    group.extend(deepcopy(list(children)))
    return group


def text_paths(text, x, y, size, color='#F8EAD5'):
    group = node('g', transform=f'translate({x} {y})')
    offset = 0
    scale = size / font['head'].unitsPerEm
    for char in text:
        glyph = glyphs[cmap[ord(char)]]
        pen = SVGPathPen(glyphs)
        glyph.draw(pen)
        g = node('g', transform=f'translate({offset:.4f} {size}) scale({scale:.6f} {-scale:.6f})')
        g.append(node('path', fill=color, d=pen.getCommands()))
        group.append(g)
        offset += glyph.width * scale
    return group


def main():
    emblem = ET.parse(HERE / 'emblem.svg').getroot()
    horizontal = ET.parse(HERE / 'horizontal.svg').getroot()
    name = ET.parse(HERE / 'full-name.svg').getroot()[1]
    for direction in ['rose-wall', 'threshold', 'atlas']:
        chosen = direction == 'rose-wall'
        prefix = '' if chosen else '../'
        root = node('svg', width=1280, height=640, viewBox='0 0 1280 640')
        root.append(node('image', href=f'{prefix}social-preview-layers/{direction}.png',
                         x=0, y=0, width=1280, height=640))
        # A quiet darkened wall preserves legibility without a floating plaque.
        defs = node('defs')
        fade = node('linearGradient', id='copy-fade')
        for offset, opacity in [('0', '1'), ('.72', '1'), ('1', '0')]:
            fade.append(node('stop', offset=offset, **{'stop-color': '#120C0D', 'stop-opacity': opacity}))
        defs.append(fade)
        root.append(defs)
        shade = node('g', id='copy-shade')
        shade.append(node('path', fill='url(#copy-fade)', opacity='.6' if direction == 'atlas' else '.36', d='M0 0H704V640H0Z'))
        root.append(shade)
        logo = node('g', id='production-logo')
        if chosen:
            # Exact emblem contours inside the scene's octagonal bronze recess.
            logo.append(placed(emblem, 802, 93, 1.328125))
            logo.append(placed(horizontal[1], 112, 174, 2.4))
        else:
            logo.append(placed(horizontal, 96, 148, 1.5))
        logo.append(placed(name, 112 if chosen else 96, 354, 1))
        root.append(logo)
        copy = node('g', id='tagline')
        x = 112 if chosen else 96
        copy.append(text_paths('Curated DOOM mapsets', x, 424, 34))
        copy.append(text_paths('for Steam Deck.', x, 466, 34))
        root.append(copy)
        # Small bronze rule ties the copy block to the architectural trim.
        root.append(node('path', fill='#B88960', d=f'M{x} 406H{x+72}V408H{x}Z'))
        out = HERE / ('social-preview.svg' if chosen else f'social-preview-variants/{direction}.svg')
        out.write_bytes(ET.tostring(root, encoding='utf-8') + b'\n')


if __name__ == '__main__':
    main()
