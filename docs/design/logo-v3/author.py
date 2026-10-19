"""Author the three Rose Compass v3 variants, their contact sheets and header mocks.

Design time only: Python 3 with fonttools==4.60.1 (labels and mock copy are
outlined from the bundled OFL Barlow and Pirata One faces). Exporting the
committed SVGs needs only resvg 0.47.0 (see export.py). This file is the single
authoring source of the logo: ../logo/author.py imports its geometry to build the
production masters (spec 039). The D-023 emblem in ../logo-v2/rose-compass/ is read
for the comparison sheet and never written.
"""
from pathlib import Path
import math
import xml.etree.ElementTree as ET
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
C = 128
# Ember Cathedral core (D-020/D-021, D-023) ...
DARK, BONE, BRONZE, BLOOD, AMBER, EMBER = '#120C0D', '#F8EAD5', '#B88960', '#52272B', '#FFD078', '#B8583F'
MUTED, SURFACE, RAISED, LINE = '#D0BCA7', '#211719', '#352527', '#80533B'
# ... extended for v3: carved metal, stone and glass ramps, darkest first.
BRONZE_RAMP = ['#5C3A27', '#7E5639', '#9D7150', '#B88960', '#D1A676', '#E9C694']
BLOOD_RAMP = ['#2A1315', '#3A1B1E', '#52272B', '#6A3337']
BONE_LIT, BONE_SHADE = '#FBF1E1', '#D8C1A2'
HOT = '#FFF3D2'


# --- primitives ---------------------------------------------------------------
def num(v):
    s = f'{v:.2f}'.rstrip('0').rstrip('.')
    return '0' if s in ('-0', '') else s


def poly(points):
    return 'M' + ' '.join(f'{num(x)} {num(y)}' for x, y in points) + 'Z'


def circle(cx, cy, r):
    return (f'M{num(cx - r)} {num(cy)}a{num(r)} {num(r)} 0 1 0 {num(2 * r)} 0'
            f'a{num(r)} {num(r)} 0 1 0 {num(-2 * r)} 0Z')


def path(d, fill, even=False, opacity=None):
    return (f'<path fill="{fill}"' + (' fill-rule="evenodd"' if even else '')
            + (f' fill-opacity="{opacity}"' if opacity is not None else '') + f' d="{d}"/>')


def group(body, x=0, y=0, scale=1):
    zoom = f' scale({scale:.6g})' if scale != 1 else ''
    return f'<g transform="translate({num(x)} {num(y)}){zoom}">{body}</g>'


def svg(w, h, body, defs=''):
    defs = f'<defs>{defs}</defs>' if defs else ''
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{defs}{body}</svg>\n'


def radial(gid, stops, r, cx=C, cy=C):
    s = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
    return (f'<radialGradient id="{gid}" gradientUnits="userSpaceOnUse" cx="{cx}" cy="{cy}" '
            f'r="{r}">{s}</radialGradient>')


def linear(gid, stops, x1, y1, x2, y2):
    s = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
    return (f'<linearGradient id="{gid}" gradientUnits="userSpaceOnUse" x1="{x1}" y1="{y1}" '
            f'x2="{x2}" y2="{y2}">{s}</linearGradient>')


# --- Rose Compass geometry (256 unit emblem, centre 128,128) -------------------
def octagon(d):
    """Production outer octagon (D-023 silhouette) offset inwards by d."""
    a = 4 + d
    b = 84 + d * math.sqrt(2) - a
    return [(b, a), (256 - b, a), (256 - a, b), (256 - a, 256 - b),
            (256 - b, 256 - a), (b, 256 - a), (a, 256 - b), (a, b)]


def ring(d1, d2):
    return poly(octagon(d1)) + poly(octagon(d2))


def local(angle, t, u):
    """Point t along the spoke at angle (0 = north, clockwise) and u across it."""
    r = math.radians(angle)
    return (C + t * math.sin(r) + u * math.cos(r), C - t * math.cos(r) + u * math.sin(r))


def kite(angle, tip, shoulder, half, base):
    return [local(angle, tip, 0), local(angle, shoulder, half),
            local(angle, base, 0), local(angle, shoulder, -half)]


def offset(points, d, limit=2.5):
    """Mitred outward offset of a simple polygon (inward for negative d).
    Spikes longer than limit * d are shortened so leads stay inside the window."""
    n = len(points)
    area = sum(points[i][0] * points[(i + 1) % n][1] - points[(i + 1) % n][0] * points[i][1]
               for i in range(n))
    sign = 1 if area > 0 else -1
    normals = []
    for i in range(n):
        (x1, y1), (x2, y2) = points[i], points[(i + 1) % n]
        length = math.hypot(x2 - x1, y2 - y1)
        normals.append((sign * (y2 - y1) / length, -sign * (x2 - x1) / length))
    out = []
    for i in range(n):
        (ax, ay), (bx, by) = normals[i - 1], normals[i]
        k = d / (1 + ax * bx + ay * by)
        k *= min(1, abs(limit * d) / (abs(k) * math.hypot(ax + bx, ay + by) or 1))
        out.append((points[i][0] + k * (ax + bx), points[i][1] + k * (ay + by)))
    return out


def clip(subject, convex):
    """Sutherland-Hodgman clip of a polygon by a convex, clockwise polygon."""
    out = subject
    for i in range(len(convex)):
        (ax, ay), (bx, by) = convex[i], convex[(i + 1) % len(convex)]
        inside = lambda p: (bx - ax) * (p[1] - ay) - (by - ay) * (p[0] - ax) >= 0
        source, out = out, []
        for j in range(len(source)):
            p, q = source[j - 1], source[j]
            if inside(q) != inside(p):
                t = ((ax - p[0]) * (by - ay) - (ay - p[1]) * (bx - ax)) / \
                    ((q[0] - p[0]) * (by - ay) - (q[1] - p[1]) * (bx - ax))
                out.append((p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])))
            if inside(q):
                out.append(q)
    return out


def torus(r1, r2, ramp, n=16):
    """A round moulding cut into n sectors, lit from the upper left."""
    fills = {}
    for k in range(n):
        a1, a2 = k * 360 / n, (k + 1) * 360 / n
        (x1, y1), (x2, y2) = local(a1, r2, 0), local(a2, r2, 0)
        (x3, y3), (x4, y4) = local(a2, r1, 0), local(a1, r1, 0)
        d = (f'M{num(x1)} {num(y1)}A{r2} {r2} 0 0 1 {num(x2)} {num(y2)}L{num(x3)} {num(y3)}'
             f'A{r1} {r1} 0 0 0 {num(x4)} {num(y4)}Z')
        fills.setdefault(shade(ramp, math.cos(math.radians((a1 + a2) / 2 - 315))), []).append(d)
    return ''.join(path(''.join(ds), col) for col, ds in fills.items())


def shade(ramp, light):
    return ramp[round((light + 1) / 2 * (len(ramp) - 1))]


def facets(d1, d2, ramp, inward=False):
    """Octagonal moulding cut into eight planes lit from the upper left."""
    outer, inner, fills = octagon(d1), octagon(d2), {}
    for k in range(8):
        light = math.cos(math.radians(k * 45 + (180 if inward else 0) - 315))
        fills.setdefault(shade(ramp, light), []).append(
            poly([outer[k], outer[(k + 1) % 8], inner[(k + 1) % 8], inner[k]]))
    return ''.join(path(''.join(ds), col) for col, ds in fills.items())


def blade(angles, tip, shoulder, half, base, lit, dark, lead=None, lead_width=0):
    """Compass points split along their spine; the counter-clockwise half is lit."""
    out = ''
    if lead:
        out += path(''.join(poly(offset(kite(a, tip, shoulder, half, base), lead_width))
                            for a in angles), lead)
    lit_d = dark_d = ''
    for a in angles:
        t, right, b, left = kite(a, tip, shoulder, half, base)
        lit_d += poly([t, b, left])
        dark_d += poly([t, right, b])
    return out + path(lit_d, lit) + path(dark_d, dark)


def gem(r, ramp_lit, ramp_dark, core=None):
    """Centre diamond cut into four facets; NW facet brightest."""
    n, e, s, w = local(0, r, 0), local(90, r, 0), local(180, r, 0), local(270, r, 0)
    out = path(poly([n, (C, C), w]), ramp_lit[0]) + path(poly([n, e, (C, C)]), ramp_lit[1])
    out += path(poly([w, (C, C), s]), ramp_dark[0]) + path(poly([(C, C), e, s]), ramp_dark[1])
    return out + (diamond(core, HOT) if core else '')


def diamond(r, fill, cx=C, cy=C):
    return path(poly([(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)]), fill)


CARDINAL, ORDINAL = (0, 90, 180, 270), (45, 135, 225, 315)


# --- Variant A: Carved Rose — ornament through carving, flat fills only -------
def carved_full():
    b = path(poly(octagon(0)), DARK)
    b += facets(4, 12, BRONZE_RAMP) + facets(12, 20, BRONZE_RAMP, inward=True)
    b += path(''.join(circle(x, y, 3.2) for x, y in octagon(12)), '#F2D6A8')
    b += path(ring(20, 23), DARK, True)
    b += facets(23, 30, BLOOD_RAMP, inward=True) + path(poly(octagon(30)), BLOOD)
    # Tracery: a carved ring with eight bronze bosses between the points.
    b += path(circle(C, C, 84) + circle(C, C, 73), DARK, True) + torus(75.5, 81.5, BRONZE_RAMP[:5])
    bosses = [a + 22.5 for a in range(0, 360, 45)]
    b += path(''.join(circle(*local(a, 78.5, 0), 6) for a in bosses), DARK)
    b += path(''.join(circle(*local(a, 78.5, 0), 4) for a in bosses), BRONZE_RAMP[5])
    b += blade(ORDINAL, 70, 50, 10, 40, BRONZE_RAMP[5], BRONZE_RAMP[2], DARK, 4)
    b += blade(CARDINAL, 88, 40, 17, 30, BONE_LIT, BONE_SHADE, DARK, 5)
    b += diamond(24, DARK) + gem(19, ['#FFE4A0', '#FFC864'], ['#E08A3E', '#B8583F'], core=6)
    return '', b


def carved_small():
    b = path(poly(octagon(0)), DARK)
    b += facets(8, 26, BRONZE_RAMP[1:])
    b += path(poly(octagon(26)), BLOOD)
    b += blade(ORDINAL, 86, 72, 11, 60, BRONZE_RAMP[5], BRONZE_RAMP[2])
    # Production leads and petals (D-023) keep the small silhouette identical.
    b += path(''.join(poly(kite(a, 86, 34, 29, 18)) for a in CARDINAL), DARK)
    b += blade(CARDINAL, 72, 38, 17, 29, BONE_LIT, BONE_SHADE)
    b += diamond(21, EMBER) + diamond(15, AMBER)
    return '', b


# --- Variant B: Stained Glass — colour through lit glass panes ----------------
def arc(r, a1, a2, steps=4):
    return [local(a1 + (a2 - a1) * i / steps, r, 0) for i in range(steps + 1)]


def lancet(angle, base, shoulder, tip, half):
    """A pointed-arch light pointing outwards along `angle`."""
    return [local(angle, base, -half), local(angle, base, half), local(angle, shoulder, half),
            local(angle, (shoulder + tip) / 2, half * 0.72), local(angle, tip, 0),
            local(angle, (shoulder + tip) / 2, -half * 0.72), local(angle, shoulder, -half)]


def glass_panes(wall, came):
    """The rose window's glass, cut by a lead-came network.

    Per 45-degree bay: one inner quarry, a ring of lead,
    an amber pointed lancet on the bisector and two ruby side panes that meet
    in a lead running out to the octagon's corner. Cells are inset by half the
    came width, so the dark ground between them reads as continuous lead.
    """
    panes = {'amber': '', 'warm': '', 'ruby': ''}
    for k in range(8):
        a1, a2 = k * 45, k * 45 + 45
        m = a1 + 22.5
        panes['warm'] += poly(offset(arc(15, a1, a2, 4) + arc(58, a2, a1, 6), -came))
        light = lancet(m, 64, 82, 96, 9)
        panes['amber'] += poly(offset(light, -came))
        left = arc(62, a1, m - 7.5, 3) + [light[0], light[6], light[5], light[4]]
        left += [local(m, 300, 0), local(a1, 300, 0)]
        right = [local(a2, 300, 0), local(m, 300, 0), light[4], light[3], light[2], light[1]]
        right += arc(62, m + 7.5, a2, 3)
        for side in (left, right):
            panes['ruby'] += poly(offset(clip(side, wall), -came))
    return panes


def glass_full(p='gl'):
    defs = linear(f'{p}-metal', [(0, '#EDCB98'), (0.45, '#B88960'), (1, '#5C3A27')], 40, 20, 216, 236)
    defs += radial(f'{p}-warm', [(0, '#E0703A'), (0.5, '#B83A2C'), (1, '#7A2026')], 60)
    defs += radial(f'{p}-ruby', [(0.55, '#B23A2E'), (0.8, '#7E2027'), (1, '#43161B')], 106)
    defs += radial(f'{p}-amber', [(0.55, '#E7A246'), (0.8, '#B8682A'), (1, '#7A3E1C')], 100)
    defs += radial(f'{p}-ember', [(0, HOT), (0.45, AMBER), (1, '#E2853F')], 13)
    b = path(poly(octagon(0)), DARK)
    b += path(ring(2.5, 20), f'url(#{p}-metal)', True)
    b += path(ring(10, 12), '#F6DDB2', True, 0.55)
    b += path(''.join(circle(x, y, 3) for x, y in octagon(11)), '#3E271C')
    b += path(poly(octagon(20)), DARK)
    panes = glass_panes(octagon(26), 1.8)
    for name in ('warm', 'ruby', 'amber'):
        b += path(panes[name], f'url(#{p}-{name})')
    b += path(circle(C, C, 62) + circle(C, C, 58), DARK, True)
    # Faceted amber jewels set where the ring lead crosses the diagonal leads.
    b += blade(ORDINAL, 67, 60, 5, 53, '#FFD58A', '#B8682A', DARK, 2.2)
    b += blade(CARDINAL, 90, 38, 16, 24, BONE_LIT, BONE_SHADE, DARK, 4.5)
    b += diamond(18, DARK) + diamond(13, f'url(#{p}-ember)')
    return defs, b


def glass_small(p='gls'):
    defs = linear(f'{p}-metal', [(0, '#EDCB98'), (0.45, '#B88960'), (1, '#5C3A27')], 40, 20, 216, 236)
    defs += radial(f'{p}-ruby', [(0, '#D2553A'), (0.6, '#A02E2A'), (1, '#5A1A1E')], 104)
    defs += radial(f'{p}-amber', [(0.4, '#DA9440'), (0.75, '#A85E24'), (1, '#74391A')], 100)
    defs += radial(f'{p}-ember', [(0, HOT), (0.5, AMBER), (1, '#E08A3E')], 19)
    b = path(poly(octagon(0)), DARK) + path(ring(6, 26), f'url(#{p}-metal)', True)
    b += path(poly(octagon(26)), DARK)
    # The full tier's bays, simplified: ruby ground and one broad amber lancet each.
    wall = octagon(26)
    for k in range(8):
        a1, a2 = k * 45, k * 45 + 45
        wedge = [local(a1, 20, 4), local(a1, 300, 4), local(a2, 300, -4), local(a2, 20, -4)]
        b += path(poly(clip(wedge, wall)), f'url(#{p}-ruby)')
    lights = [lancet(a + 22.5, 50, 72, 92, 10) for a in range(0, 360, 45)]
    b += path(''.join(poly(offset(l, 4)) for l in lights), DARK)
    b += path(''.join(poly(l) for l in lights), f'url(#{p}-amber)')
    b += blade(CARDINAL, 84, 36, 22, 24, BONE_LIT, BONE_SHADE, DARK, 9)
    b += diamond(23, DARK) + diamond(19, f'url(#{p}-ember)')
    return defs, b


# --- Variant C: Furnace Rose — stone tracery backlit by the ember -------------
def furnace_full(p='fu'):
    defs = radial(f'{p}-glow', [(0, HOT), (0.14, AMBER), (0.36, '#F08A3C'), (0.62, '#C8452B'),
                                (0.85, '#7A2422'), (1, '#3A1315')], 108)
    defs += radial(f'{p}-stone', [(0, '#4A2620'), (0.45, '#2C1718'), (1, '#160E0F')], 100)
    b = path(poly(octagon(0)), DARK)
    b += facets(4, 12, BRONZE_RAMP) + facets(12, 20, BRONZE_RAMP, inward=True)
    b += path(ring(20, 22.5), '#D9773F', True)
    b += path(poly(octagon(22.5)), f'url(#{p}-glow)')
    # A carved stone plate: the compass star and the oculus are cut through it,
    # each opening ringed by a chamfer that catches the furnace light.
    holes = [kite(a, 72, 36, 16, 26) for a in CARDINAL] + [kite(a, 60, 46, 9, 38) for a in ORDINAL]
    b += path(poly(octagon(30)) + ''.join(poly(offset(h, 3)) for h in holes) + circle(C, C, 19),
              f'url(#{p}-stone)', True)
    b += path(''.join(poly(offset(h, 3)) + poly(h) for h in holes) + circle(C, C, 19) + circle(C, C, 16),
              '#8A3A26', True)
    b += path(circle(C, C, 85) + circle(C, C, 77), DARK, True) + torus(78.5, 83.5, BRONZE_RAMP[:5])
    bosses = [local(a, 81, 0) for a in range(0, 360, 45)]
    b += path(''.join(circle(x, y, 5.5) for x, y in bosses), DARK)
    b += path(''.join(circle(x, y, 3.5) for x, y in bosses), BRONZE_RAMP[5])
    # The clockwise half of every blade sits in the stone's shadow.
    b += path(''.join(poly(h[:3]) for h in holes), '#4A0F12', opacity=0.5)
    return defs, b


def furnace_small(p='fus'):
    defs = radial(f'{p}-glow', [(0, HOT), (0.25, AMBER), (0.6, '#F59A45'), (0.9, '#D2502E'),
                                (1, '#9C2E26')], 96)
    b = path(poly(octagon(0)), DARK) + facets(8, 22, BRONZE_RAMP[1:])
    b += path(poly(octagon(22)), f'url(#{p}-glow)')
    holes = [kite(a, 84, 38, 26, 28) for a in CARDINAL]
    b += path(poly(octagon(27)) + ''.join(poly(h) for h in holes) + circle(C, C, 18), '#241415', True)
    b += path(''.join(poly(h[:3]) for h in holes), '#4A0F12', opacity=0.5)
    return defs, b


# --- DDL wordmark (D-023 contours) with carved depth ---------------------------
D_OUTER = [(0, 0), (25, 0), (42, 15), (42, 49), (25, 64), (0, 64)]
D_COUNTER = [(13, 13), (13, 51), (21, 51), (29, 43), (29, 21), (21, 13)]
L_SHAPE = [(108, 0), (122, 0), (122, 50), (154, 50), (154, 64), (108, 64)]


def shifted(points, dx, dy=0):
    return [(x + dx, y + dy) for x, y in points]


def area(points):
    n = len(points)
    return sum(points[i][0] * points[(i + 1) % n][1] - points[(i + 1) % n][0] * points[i][1]
               for i in range(n))


def hull(points):
    """Convex hull (monotone chain), wound like octagon() so clip() accepts it."""
    pts = sorted(set(points))
    cross = lambda o, a, b: (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    lower, upper = [], []
    for p in pts:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], p) <= 0:
            lower.pop()
        lower.append(p)
    for p in reversed(pts):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], p) <= 0:
            upper.pop()
        upper.append(p)
    out = lower[:-1] + upper[:-1]
    return out if area(out) * area(octagon(0)) > 0 else out[::-1]


DEPTH = (3, 3)


def solid(convex, dx=0):
    """A convex letter part swept back along DEPTH: one solid, no seams."""
    return hull(shifted(convex, dx) + shifted(convex, dx + DEPTH[0], DEPTH[1]))


def tunnel(counter, dx=0):
    """Where a counter stays open through the whole depth."""
    return clip(shifted(counter, dx), hull(shifted(counter, dx + DEPTH[0], DEPTH[1])))


def letters(grow=0, swept=False):
    """DDL contours, grown by `grow`; swept=True gives the extruded solid."""
    ds = ''
    for x in (0, 54):
        outer, counter = offset(D_OUTER, grow), offset(D_COUNTER, -grow)
        ds += poly(solid(outer, x) if swept else shifted(outer, x))
        ds += poly(tunnel(counter, x) if swept else shifted(counter, x))
    # The L as two convex bars so the sweep stays exact.
    bars = [[(108 - grow, -grow), (122 + grow, -grow), (122 + grow, 64 + grow), (108 - grow, 64 + grow)],
            [(108 - grow, 50 - grow), (154 + grow, 50 - grow), (154 + grow, 64 + grow), (108 - grow, 64 + grow)]]
    return ds, ''.join(poly(hull(b + shifted(b, *DEPTH)) if swept else b) for b in bars)


def wordmark(face, depth):
    """Solid dark outline for light grounds, a carved side in depth colour, bone face."""
    body = ''
    for grow, fill in ((3.5, DARK), (0, depth)):
        ds, l = letters(grow, swept=True)
        body += path(ds, fill, True) + path(l, fill)
    ds, l = letters()
    return body + path(ds, face, True) + path(l, face)


VARIANTS = {
    'carved-rose': dict(title='A · Carved Rose', full=carved_full, small=carved_small,
                        ddl=lambda: ('', wordmark(BONE, BRONZE_RAMP[1]))),
    'stained-glass': dict(title='B · Stained Glass', full=glass_full, small=glass_small,
                          ddl=lambda: ('', wordmark(BONE, '#A8302A'))),
    'furnace-rose': dict(title='C · Furnace Rose', full=furnace_full, small=furnace_small,
                         ddl=lambda: (linear('fuw-face', [(0, BONE), (0.62, BONE), (1, AMBER)], 0, 0, 0, 64),
                                      wordmark('url(#fuw-face)', '#9C2E26'))),
}


# --- outlined copy (bundled OFL faces, never system fonts) ----------------------
FONTS = {name: TTFont(ROOT / f'crates/ddl/ui/assets/{name}.ttf') for name in ('Barlow-Regular', 'PirataOne-Regular')}


def lettering(text, size, color, face='Barlow-Regular'):
    font = FONTS[face]
    glyphs, cmap, scale = font.getGlyphSet(), font.getBestCmap(), size / font['head'].unitsPerEm
    pen = SVGPathPen(glyphs, ntos=lambda v: num(v))
    x = 0
    for char in text:
        glyph = glyphs[cmap[ord(char)]]
        glyph.draw(TransformPen(pen, (scale, 0, 0, -scale, x, size)))
        x += glyph.width * scale
    return path(pen.getCommands(), color), x


def label(text, x, y, size=18, color=MUTED, face='Barlow-Regular', centre=False):
    body, width = lettering(text, size, color, face)
    return group(body, x - width / 2 if centre else x, y)


# --- compositions ---------------------------------------------------------------
def lockup(small, ddl):
    """Same 330x128 frame as the production horizontal master (header drop-in)."""
    # A dark tablet with the octagon's 45-degree cuts: invisible on the dark
    # header, and on light grounds it keeps the bone letters from bleeding out.
    # Kept tight to the letters (about 8 units of margin) so it weighs little
    # beside the emblem on light grounds.
    tablet = path(poly([(149, 19), (315, 19), (323, 27), (323, 101), (315, 109), (149, 109),
                        (141, 101), (141, 27)]), DARK)
    return small[0] + ddl[0], group(small[1], 0, 0, 0.5) + tablet + group(ddl[1], 152, 30)


def header_mock(lock):
    """1280x800 app shell: header geometry copied from app-window.slint."""
    b = path('M0 0H1280V800H0Z', '#1B1214')
    b += ''.join(path(f'M{x} {y}h{w}v{h}h{-w}Z', '#22171A') for x, y, w, h in
                 [(0, 104, 1280, 632), (0, 104, 300, 300), (620, 260, 660, 220)])
    b += path('M0 0H1280V104H0Z', DARK)
    scale = min(154 / 330, 56 / 128)
    b += group(lock, 40 + (154 - 330 * scale) / 2, 24 + (56 - 128 * scale) / 2, scale)
    for i, tab in enumerate(['Home', 'Discover', 'Library', 'Settings']):
        x = 214 + i * 180
        if i == 2:
            b += path(f'M{x + 8} 24h132a8 8 0 0 1 8 8v36a8 8 0 0 1-8 8h-132a8 8 0 0 1-8-8v-36a8 8 0 0 1 8-8Z', RAISED)
            b += path(f'M{x + 24} 80h100v3h-100Z', AMBER)
        text, width = lettering(tab, 22, BONE)
        b += group(text, x + (148 - width) / 2, 36)
    b += path('M40 102h1200v1h-1200Z', LINE)
    b += path('M40 120h1200v88h-1200Z', DARK)
    b += label('Library', 40, 128, 36, BONE, 'PirataOne-Regular')
    b += label('Installed mapsets and base games', 40, 176, 22, MUTED)
    for i in range(4):
        x = 48 + i * 300
        b += path(f'M{x} 240h276v380h-276Z', SURFACE) + path(f'M{x} 240h276v2h-276Z', LINE)
        b += path(f'M{x + 12} 252h252v200h-252Z', RAISED)
        b += path(f'M{x + 12} 470h180v18h-180Z M{x + 12} 500h120v14h-120Z', '#3E2C2E')
    b += path('M44 236h284v388h-284Z M48 240v380h276v-380Z', AMBER, True)
    b += path('M0 736H1280V800H0Z', DARK) + path('M0 736H1280v1H0Z', LINE)
    b += label('A  Select     B  Back     Y  Search', 40, 754, 20, MUTED)
    return b


def contact_sheet(slug, spec):
    full, small = spec['full'](), spec['small']()
    lock = lockup(small, spec['ddl']())
    defs = full[0] + small[0] + lock[0]
    b = path('M0 0H1440V2528H0Z', '#1A1213')
    b += label(f'Rose Compass v3 — {spec["title"]}', 40, 24, 30, BONE)
    b += label('Native scale. Small tier for 32–64 px (app and window icon, header); '
               'full tier from 128 px. Full tier at 32–64 px shown only as a reference.', 40, 66, 18)
    for row, (bg, ink) in enumerate([(DARK, MUTED), (BONE, '#4A3A33')]):
        y0 = 104 + row * 640
        b += path(f'M0 {y0}H1440V{y0 + 640}H0Z', bg)
        b += label('small tier', 40, y0 + 24, 18, ink) + label('full tier (reference)', 40, y0 + 238, 18, ink)
        x = 40
        for size in (32, 48, 64):
            b += group(small[1], x, y0 + 64 + 64 - size, size / 256)
            b += group(full[1], x, y0 + 278 + 64 - size, size / 256)
            b += label(str(size), x + size / 2, y0 + 140, 16, ink, centre=True)
            b += label(str(size), x + size / 2, y0 + 354, 16, ink, centre=True)
            x += size + 40
        for size, x in ((128, 320), (256, 488), (512, 784)):
            b += group(full[1], x, y0 + 40 + 512 - size, size / 256)
            b += label(f'{size} px', x + size / 2, y0 + 572, 18, ink, centre=True)
    y0 = 1384
    b += label('Horizontal lock-up (small-tier emblem): native 330×128 and header size 154×56', 40, y0, 18)
    for col, bg in enumerate([DARK, BONE]):
        x0 = 40 + col * 690
        b += path(f'M{x0} {y0 + 36}h650v200h-650Z', bg)
        b += group(lock[1], x0 + 30, y0 + 72)
        b += group(lock[1], x0 + 410, y0 + 72 + 36, 56 / 128)
    b += label('1280×800 app shell mock with the lock-up in the header slot', 40, y0 + 266, 18)
    b += group(header_mock(lock[1]), 80, y0 + 304)
    return svg(1440, 2528, b, defs)


def comparison():
    production = ET.parse(HERE.parent / 'logo-v2/rose-compass/emblem.svg').getroot()
    current = ''.join(ET.tostring(el, encoding='unicode') for el in production).replace('ns0:', '').replace(' xmlns:ns0="http://www.w3.org/2000/svg"', '')
    rows = [('Production (D-023)', ('', current), ('', current))]
    rows += [(spec['title'], spec['small'](), spec['full']()) for spec in VARIANTS.values()]
    defs = ''.join(small[0] + full[0] for _, small, full in rows)
    b = path('M0 0H1200V1080H0Z', DARK) + path('M600 0H1200V1080H600Z', BONE)
    for i, (title, small, full) in enumerate(rows):
        y = 24 + i * 264
        for col, ink in ((0, MUTED), (1, '#4A3A33')):
            x = 24 + col * 600
            b += label(title, x, y, 18, ink)
            b += group(small[1], x, y + 36 + 32, 32 / 256) + group(small[1], x + 56, y + 36 + 16, 48 / 256)
            b += group(small[1], x + 128, y + 36, 64 / 256) + group(full[1], x + 228, y + 36 - 8, 200 / 256)
    return svg(1200, 1080, b, defs)


def main():
    for slug, spec in VARIANTS.items():
        folder = HERE / slug
        folder.mkdir(exist_ok=True)
        full, small = spec['full'](), spec['small']()
        lock = lockup(small, spec['ddl']())
        (folder / 'emblem.svg').write_text(svg(256, 256, full[1], full[0]))
        (folder / 'emblem-small.svg').write_text(svg(256, 256, small[1], small[0]))
        (folder / 'horizontal.svg').write_text(svg(330, 128, lock[1], lock[0]))
        (folder / 'contact-sheet.svg').write_text(contact_sheet(slug, spec))
    (HERE / 'comparison.svg').write_text(comparison())


if __name__ == '__main__':
    main()
