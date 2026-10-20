"""Ember Cathedral scenery for the Steam compositions (spec 054, D-021).

Procedural vector interiors: clustered piers, moulded pointed arches, lancet and
rose glass, stone courses, braziers and layered light. Original art, drawn only
from the shapes below. The output keeps to the ../logo-v3/export.py allowlist:
paths, local linear/radial gradients and fill-opacity; soft light and shade are
stacked translucent shapes, never filters or masks. Deterministic: fixed seeds
and two-decimal coordinates, so author.py rebuilds the same bytes.

author.py places the emblem and lettering over these scenes; each scene darkens
the area its lettering uses (see the *_FIT notes on each function).
"""
import importlib.util
import math
import random
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location('logo_v3_author', HERE.parent / 'logo-v3/author.py')
v3 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(v3)
num, poly, path = v3.num, v3.poly, v3.path
DARK, BONE, AMBER, EMBER, HOT = v3.DARK, v3.BONE, v3.AMBER, v3.EMBER, v3.HOT
BRONZE_RAMP, BLOOD_RAMP = v3.BRONZE_RAMP, v3.BLOOD_RAMP
# Stone, darkest first: oxblood masonry under ember light.
STONE = ['#0E0809', '#170D0F', '#22120F', '#2E1716', '#3E1E1C', '#542A24', '#6E3A2C', '#8A5236']
LEAD = '#0B0607'
FIRE = ['#7A2422', '#B8402A', '#E2703A', '#F59A45', AMBER, HOT]


# --- primitives ---------------------------------------------------------------
def op(v):
    return f'{v:.3f}'.rstrip('0').rstrip('.')


def mix(a, b, t):
    ca, cb = bytes.fromhex(a[1:]), bytes.fromhex(b[1:])
    return '#' + ''.join(f'{round(x + (y - x) * t):02X}' for x, y in zip(ca, cb))


def rect(x, y, w, h):
    return poly([(x, y), (x + w, y), (x + w, y + h), (x, y + h)])


def ellipse(cx, cy, rx, ry):
    return (f'M{num(cx - rx)} {num(cy)}a{num(rx)} {num(ry)} 0 1 0 {num(2 * rx)} 0'
            f'a{num(rx)} {num(ry)} 0 1 0 {num(-2 * rx)} 0Z')


def circle(cx, cy, r):
    return ellipse(cx, cy, r, r)


def matrix(body, a, d, e, f):
    """Axis-aligned scale (a, d) then translate (e, f): shared unit gradients
    follow the shape, since userSpaceOnUse resolves inside the transform."""
    return f'<g transform="matrix({num(a)} 0 0 {num(d)} {num(e)} {num(f)})">{body}</g>'


class Defs:
    """Gradient definitions with stable, per-scene ids; identical gradients are shared."""

    def __init__(self, prefix):
        self.prefix, self.items, self.cache = prefix, [], {}

    def _add(self, key, make):
        if key not in self.cache:
            gid = f'{self.prefix}{len(self.cache) + 1}'
            self.items.append(make(gid))
            self.cache[key] = f'url(#{gid})'
        return self.cache[key]

    def linear(self, stops, x1, y1, x2, y2):
        args = tuple(num(v) for v in (x1, y1, x2, y2))
        return self._add(('l', tuple(stops), args), lambda gid: v3.linear(gid, stops, *args))

    def radial(self, stops, r, cx, cy):
        args = tuple(num(v) for v in (r, cx, cy))
        return self._add(('r', tuple(stops), args), lambda gid: v3.radial(gid, stops, *args))

    def svg(self):
        return ''.join(self.items)


def glow(cx, cy, rx, ry, color, alpha, steps=8):
    """Soft light or shade: stacked ellipses, densest at the centre. Each step is
    split in three fainter layers so the rings stay below one 8-bit level."""
    n = steps * 3
    return ''.join(path(ellipse(cx, cy, rx * (1 - i / n), ry * (1 - i / n)), color, opacity=op(alpha / 3))
                   for i in range(n))


def gloom(w, depth, alpha, steps=8):
    """Darkness gathering in the vault: stacked bands from the top edge, no hard line."""
    n = steps * 3
    return ''.join(path(rect(0, 0, w, depth * (1 - i / n)), DARK, opacity=op(alpha / 3)) for i in range(n))


def vignette(w, h, cx, cy, rx, ry, color, alpha, steps=8, grow=1.6):
    """Shade outside a widening ellipse; the frame edge receives every layer."""
    frame, n = rect(0, 0, w, h), steps * 3
    return ''.join(path(frame + ellipse(cx, cy, rx * (1 + (grow - 1) * i / n), ry * (1 + (grow - 1) * i / n)),
                        color, True, op(alpha / 3)) for i in range(n))


# --- gothic geometry ------------------------------------------------------------
def arch(cx, base, spring, half, k=2.0, n=12):
    """Pointed opening: jambs from base to spring, then two arcs of radius k * half
    meeting at the apex (k = 2 is the equilateral arch). Clockwise on screen."""
    r = k * half
    c = r - half
    end = math.acos(-c / r)
    left = [(cx + c + r * math.cos(t), spring - r * math.sin(t))
            for t in (math.pi - (math.pi - end) * i / n for i in range(n + 1))]
    right = [(2 * cx - x, y) for x, y in reversed(left[:-1])]
    return [(cx - half, base)] + left + right + [(cx + half, base)]


def arch_offset(cx, base, spring, half, k, d, n=12):
    """The same arch grown by d all round (concentric arcs, jambs moved out)."""
    return arch(cx, base, spring, half + d, (k * half + d) / (half + d), n)


def apex(spring, half, k):
    r = k * half
    return spring - math.sqrt(r * r - (r - half) ** 2)


def mouldings(cx, base, spring, half, k, orders):
    """Carved orders around an opening, outermost first: [(grow, fill), ...]."""
    return ''.join(path(poly(arch_offset(cx, base, spring, half, k, d)), fill) for d, fill in orders)


def quarries(shape, size, lead, rng, fills):
    """Diamond glass quarries clipped to a convex light, inset so lead shows between."""
    xs = [x for x, _ in shape]
    ys = [y for _, y in shape]
    out = {f: '' for f in fills}
    y, row = min(ys) - size, 0
    while y < max(ys) + size:
        x = min(xs) - size + (size / 2 if row % 2 else 0)
        while x < max(xs) + size:
            piece = v3.clip([(x, y - size / 2 + lead), (x + size / 2 - lead, y), (x, y + size / 2 - lead),
                             (x - size / 2 + lead, y)], shape)
            if len(piece) >= 3:
                out[fills[min(len(fills) - 1, int(rng.random() * len(fills) * 1.15))]] += poly(piece)
            x += size
        y += size / 2
        row += 1
    return ''.join(path(d, f) for f, d in out.items() if d)


def glass(defs, cx, top, bottom, heat):
    """Backlit glass: hot near the head, ember and ruby lower down, dimmed by heat (0..1)."""
    ramp = [mix(DARK, c, heat) for c in (HOT, AMBER, '#F08A3C', '#C8452B', '#8A2622')]
    lit = defs.linear([(0, ramp[0]), (0.22, ramp[1]), (0.5, ramp[2]), (0.8, ramp[3]), (1, ramp[4])],
                      cx, top, cx, bottom)
    ruby = defs.linear([(0, mix(DARK, '#E0603A', heat)), (0.5, mix(DARK, '#A8302A', heat)),
                        (1, mix(DARK, '#5A1A1E', heat))], cx, top, cx, bottom)
    return lit, ruby


def lancet(defs, cx, base, spring, half, heat, rng, lights=2, frame=STONE):
    """A moulded lancet window: splayed orders, mullioned lights in lead quarries,
    and an oculus with a quatrefoil in the head."""
    k = 2.0
    out = mouldings(cx, base, spring, half, k, [(half * .34, frame[2]), (half * .26, frame[5]),
                                               (half * .2, frame[3]), (half * .12, frame[6]),
                                               (half * .06, frame[2])])
    opening = arch(cx, base, spring, half, k)
    out += path(poly(opening), LEAD)
    top = apex(spring, half, k)
    lit, ruby = glass(defs, cx, top, base, heat)
    q = max(6, half * .3)
    if lights == 1:
        out += quarries(arch(cx, base, spring, half * .9, k), q, max(.6, q * .07), rng, [lit, lit, ruby])
        return out
    m = half * .12
    sub = (half - m) / 2
    for side in (-1, 1):
        sx = cx + side * (sub + m / 2)
        sub_spring = spring + half * .25
        light = arch(sx, base - half * .04, sub_spring, sub * .86, 2.0)
        out += quarries(light, q, max(.6, q * .07), rng, [lit, lit, ruby])
    # Oculus between the sub-arch heads, with a quatrefoil of glass.
    oy, orad = top + (spring - top) * .52, half * .42
    out += path(circle(cx, oy, orad), frame[5]) + path(circle(cx, oy, orad * .86), LEAD)
    foil = ''.join(circle(cx + orad * .38 * dx, oy + orad * .38 * dy, orad * .36)
                   for dx, dy in ((0, -1), (1, 0), (0, 1), (-1, 0)))
    out += path(foil, lit) + path(circle(cx, oy, orad * .16), frame[6])
    return out


def rose(defs, cx, cy, r, heat, n=12):
    """Rose window: moulded rim, a ring of petal lights, outer oculi, central eye."""
    out = path(circle(cx, cy, r * 1.16), STONE[2]) + path(circle(cx, cy, r * 1.1), STONE[6])
    out += path(circle(cx, cy, r * 1.06), STONE[3]) + path(circle(cx, cy, r * 1.02), STONE[5])
    out += path(circle(cx, cy, r), LEAD)
    ramp = [mix(DARK, c, heat) for c in (HOT, AMBER, '#F08A3C', '#C8452B', '#7A2026')]
    hot = defs.radial([(0, ramp[0]), (0.25, ramp[1]), (0.55, ramp[2]), (0.8, ramp[3]), (1, ramp[4])], r, cx, cy)
    ruby = defs.radial([(0, mix(DARK, '#E0603A', heat)), (0.6, mix(DARK, '#A8302A', heat)),
                        (1, mix(DARK, '#4A161B', heat))], r, cx, cy)

    def at(a, t, u=0):
        s, c = math.sin(math.radians(a)), math.cos(math.radians(a))
        return (cx + t * s + u * c, cy - t * c + u * s)

    petals = oculi = cusps = ''
    for i in range(n):
        a = i * 360 / n
        w = r * math.sin(math.pi / n) * .62
        petals += poly([at(a, r * .27, -w * .45), at(a, r * .27, w * .45), at(a, r * .62, w),
                        at(a, r * .74, w * .7), at(a, r * .82, 0), at(a, r * .74, -w * .7), at(a, r * .62, -w)])
        b = a + 180 / n
        oculi += circle(*at(b, r * .86), r * .085)
        cusps += circle(*at(b, r * .58), r * .05)
    out += path(petals, hot) + path(oculi, ruby) + path(cusps, ruby)
    out += path(circle(cx, cy, r * .2), STONE[5]) + path(circle(cx, cy, r * .16), hot)
    out += path(circle(cx, cy, r * .05), STONE[6])
    return out


def pier(defs, x, top, bottom, w, light, cap=True):
    """Clustered pier in elevation: a core with four engaged shafts, a foliate
    capital and a moulded base. light: -1 lit from the left, 1 from the right,
    scaled by its magnitude (0..1)."""
    mag = abs(light)
    lit = mix(STONE[3], BRONZE_RAMP[3], mag * .75)
    stops = [(0, STONE[0]), (.3, STONE[2]), (.62, lit), (.78, mix(STONE[4], lit, .4)), (1, STONE[0])]
    if light < 0:
        stops = [(round(1 - o, 2), c) for o, c in reversed(stops)]
    shaft = defs.linear(stops, 0, 0, 1, 0)
    core = defs.linear([(0, STONE[1]), (.5, mix(STONE[2], lit, .25)), (1, STONE[1])] if light < 0 else
                       [(0, STONE[1]), (.5, mix(STONE[2], lit, .25)), (1, STONE[1])], 0, 0, 1, 0)
    h = bottom - top
    out = matrix(path(rect(0, 0, 1, 1), core), w * .78, h, x - w * .39, top)
    for off, sw in ((-.42, .16), (.42, .16), (-.17, .3), (.17, .3)):
        out += matrix(path(rect(0, 0, 1, 1), shaft), w * sw, h, x + w * off - w * sw / 2, top)
    # Fillets between the shafts catch a thin line of light.
    for off in (-.32, .32):
        out += path(rect(x + w * off - w * .012, top, w * .024, h), lit, opacity=op(.35 * mag + .1))
    if cap:
        out += capital(defs, x, top, w, light)
    out += base_of(x, bottom, w, light)
    return out


def capital(defs, x, y, w, light):
    lit = mix(STONE[4], BRONZE_RAMP[4], abs(light) * .8)
    out = path(rect(x - w * .62, y - w * .26, w * 1.24, w * .1), lit)
    out += path(rect(x - w * .62, y - w * .16, w * 1.24, w * .03), STONE[0])
    out += path(poly([(x - w * .58, y - w * .13), (x + w * .58, y - w * .13), (x + w * .44, y + w * .06),
                      (x - w * .44, y + w * .06)]), STONE[3])
    leaves = ''
    for i in range(7):
        lx = x - w * .45 + i * w * .15
        leaves += (f'M{num(lx - w * .06)} {num(y + w * .05)}C{num(lx - w * .08)} {num(y - w * .05)} '
                   f'{num(lx - w * .02)} {num(y - w * .12)} {num(lx + w * .05 * light)} {num(y - w * .14)}'
                   f'C{num(lx + w * .03)} {num(y - w * .06)} {num(lx + w * .06)} {num(y)} {num(lx + w * .06)} '
                   f'{num(y + w * .05)}Z')
    out += path(leaves, lit, opacity='0.8')
    out += path(rect(x - w * .42, y + w * .06, w * .84, w * .035), lit)
    return out


def base_of(x, y, w, light):
    lit = mix(STONE[4], BRONZE_RAMP[3], abs(light) * .6)
    out = path(rect(x - w * .6, y - w * .22, w * 1.2, w * .22), STONE[2])
    out += path(rect(x - w * .6, y - w * .22, w * 1.2, w * .03), lit)
    out += path(rect(x - w * .5, y - w * .32, w, w * .1), STONE[3])
    out += path(rect(x - w * .5, y - w * .32, w, w * .025), lit)
    return out


def masonry(x0, y0, x1, y1, course, block, rng, light=EMBER):
    """Stone courses: staggered blocks, random weathering, lit top arrises."""
    joints = dark = warm = arris = ''
    jw = max(.8, course * .07)
    y, row = y0, 0
    while y < y1:
        joints += rect(x0, y, x1 - x0, jw)
        x = x0 - block * (.5 if row % 2 else 0) - rng.random() * block * .3
        while x < x1:
            w = block * (.75 + .5 * rng.random())
            joints += rect(x, y, jw, course)
            arris += rect(x + jw, y + jw, w - jw, jw * .7)
            v = rng.random()
            if v < .28:
                dark += rect(x + jw, y + jw, w - jw, course - jw)
            elif v > .86:
                warm += rect(x + jw, y + jw, w - jw, course - jw)
            x += w
        y += course
        row += 1
    return (path(dark, DARK, opacity='0.28') + path(warm, light, opacity='0.08')
            + path(arris, BRONZE_RAMP[4], opacity='0.07') + path(joints, LEAD, opacity='0.6'))


def flame(x, y, wb, h, lean):
    return (f'M{num(x - wb)} {num(y)}C{num(x - wb)} {num(y - h * .45)} {num(x + lean - wb * .3)} '
            f'{num(y - h * .7)} {num(x + lean)} {num(y - h)}C{num(x + lean + wb * .15)} {num(y - h * .65)} '
            f'{num(x + wb)} {num(y - h * .45)} {num(x + wb)} {num(y)}C{num(x + wb * .6)} {num(y + wb * .5)} '
            f'{num(x - wb * .6)} {num(y + wb * .5)} {num(x - wb)} {num(y)}Z')


def brazier(defs, cx, floor, s, rng):
    """Bronze tripod brazier with a layered fire and its light."""
    out = glow(cx, floor - s * 1.6, s * 4.2, s * 3.4, '#E2703A', .045, 10)
    out += glow(cx, floor + s * .05, s * 2.6, s * .35, '#E2703A', .07, 6)
    top = floor - s * 1.25
    legs = ''
    for dx in (-1, 0, 1):
        fx = cx + dx * s * .75
        legs += poly([(cx + dx * s * .35 - s * .05, top + s * .3), (cx + dx * s * .35 + s * .05, top + s * .3),
                      (fx + s * .04, floor), (fx - s * .04, floor)])
    out += path(legs, BRONZE_RAMP[0])
    out += path(poly([(cx - s * .14, top + s * .3), (cx + s * .14, top + s * .3), (cx + s * .1, top + s * .62),
                      (cx - s * .1, top + s * .62)]), BRONZE_RAMP[1])
    bowl = defs.linear([(0, BRONZE_RAMP[5]), (.35, BRONZE_RAMP[3]), (1, BRONZE_RAMP[0])],
                       cx - s * .6, top - s * .1, cx + s * .5, top + s * .45)
    out += path(f'M{num(cx - s * .62)} {num(top)}A{num(s * .62)} {num(s * .42)} 0 0 0 {num(cx + s * .62)} '
                f'{num(top)}Z', bowl)
    out += path(rect(cx - s * .66, top - s * .06, s * 1.32, s * .07), BRONZE_RAMP[4])
    out += path(ellipse(cx, top - s * .04, s * .56, s * .09), '#7A2422')
    for i, col in enumerate(FIRE[1:]):
        k = 1 - i * .17
        fl = ''
        for j, (dx, hh, ln) in enumerate(((-.3, .9, -.12), (0, 1.35, .05), (.28, 1, .14), (-.12, .7, .2))):
            fl += flame(cx + dx * s, top - s * .02, s * .2 * k, s * hh * k, s * ln)
        out += path(fl, col)
    sparks = ''
    for _ in range(9):
        sx, sy = cx + (rng.random() - .5) * s * 1.6, top - s * (1.2 + rng.random() * 1.8)
        r = s * (.015 + rng.random() * .025)
        sparks += poly([(sx, sy - r * 2), (sx + r, sy), (sx, sy + r * 2), (sx - r, sy)])
    return out + path(sparks, AMBER, opacity='0.85')


def shaft_of_light(x_top, y_top, w_top, x_bot, y_bot, w_bot, alpha, layers=4):
    return ''.join(path(poly([(x_top - w_top * k, y_top), (x_top + w_top * k, y_top),
                              (x_bot + w_bot * k, y_bot), (x_bot - w_bot * k, y_bot)]), '#F08A3C', opacity=op(alpha))
                   for k in (1 - i / (layers + 1) for i in range(layers)))


def floor_tiles(defs, x0, x1, y0, y1, vx, vy, rng, glow_x, span):
    """Perspective flagstones converging on (vx, vy), lit by a warm pool at glow_x."""
    base = defs.linear([(0, STONE[3]), (1, STONE[1])], 0, y0, 0, y1)
    out = path(rect(x0, y0, x1 - x0, y1 - y0), base)
    out += glow(glow_x, y0 + (y1 - y0) * .25, span, (y1 - y0) * .7, '#C8452B', .05, 8)
    joints = ''
    tile = span * .2
    for i in range(-60, 61):
        bx = vx + i * tile
        tx = vx + (bx - vx) * (y0 - vy) / (y1 - vy)
        if max(tx, bx) < x0 - tile or min(tx, bx) > x1 + tile:
            continue
        joints += poly([(tx - .5, y0), (tx + .5, y0), (bx + 1.4, y1), (bx - 1.4, y1)])
    z = 1.0
    while True:
        y = vy + (y0 - vy) * z
        if y > y1:
            break
        joints += rect(x0, y, x1 - x0, max(.8, (y - vy) * .012))
        z *= 1.22
    return out + path(joints, LEAD, opacity='0.55')


def embers(region, n, rng, scale=1):
    x0, y0, x1, y1 = region
    out = ''
    for _ in range(n):
        x, y = x0 + rng.random() * (x1 - x0), y0 + rng.random() * (y1 - y0)
        r = scale * (.6 + rng.random() * 1.6)
        out += poly([(x, y - r * 2.2), (x + r, y), (x, y + r * 2.2), (x - r, y)])
    return path(out, AMBER, opacity='0.7')


def octagon_recess(cx, cy, size):
    """Carved octagonal recess for the emblem: chamfered stone, a bronze bead and a
    dark bed, lit from the upper left like the emblem's own frame (D-029)."""
    s = size / 256
    body = v3.facets(-46, -32, STONE[1:7], inward=True) + v3.facets(-32, -28, BRONZE_RAMP[:4])
    body += v3.facets(-26, -14, STONE[0:6], inward=True) + path(poly(v3.octagon(-14)), LEAD)
    return f'<g transform="translate({num(cx - size / 2)} {num(cy - size / 2)}) scale({s:.6g})">{body}</g>'


def roundel(cx, cy, r):
    """Carved spandrel medallion: moulded rings around a sunk quatrefoil and a bronze boss."""
    out = path(circle(cx, cy, r), STONE[1]) + path(circle(cx - r * .03, cy - r * .03, r * .93), STONE[5])
    out += path(circle(cx, cy, r * .88), STONE[2]) + path(circle(cx, cy, r * .8), STONE[4])
    foil = lambda k, dx=0, dy=0: ''.join(circle(cx + dx + r * .3 * ux, cy + dy + r * .3 * uy, r * .3 * k)
                                       for ux, uy in ((0, -1), (1, 0), (0, 1), (-1, 0)))
    out += path(foil(1.06, r * .03, r * .03), STONE[6]) + path(foil(1), STONE[0])
    out += path(circle(cx, cy, r * .14), BRONZE_RAMP[1]) + path(circle(cx - r * .03, cy - r * .03, r * .09), BRONZE_RAMP[4])
    return out


def dogtooth(cx, base, spring, half, k, d, size, fill, step=3):
    """Small pyramidal studs along an arch order: lit and shaded halves."""
    pts = arch_offset(cx, base, spring, half, k, d, 48)[1:-1]
    lit = shade = ''
    for x, y in pts[::step]:
        lit += poly([(x, y - size), (x - size, y), (x + size, y)])
        shade += poly([(x - size, y), (x, y + size), (x + size, y)])
    return path(lit, fill) + path(shade, STONE[1])


# --- compositions -----------------------------------------------------------------
def hero():
    """3840x1240, no mark and no text. The glowing apse sits in Steam's centred
    safe area; the side bays fall into shadow so any crop keeps the focus."""
    W, H = 3840, 1240
    d, rng = Defs('hr'), random.Random(54)
    cx, floor, vy = W / 2, 1090, 700
    wall = d.radial([(0, STONE[6]), (.25, STONE[4]), (.6, STONE[2]), (1, STONE[0])], 2100, cx, 620)
    b = path(rect(0, 0, W, H), wall) + masonry(0, -10, W, floor, 38, 112, rng)
    # Side bays: blind arcades with lancets that dim away from the centre.
    piers = [(60, 120), (700, 130), (1340, 170), (2500, 170), (3140, 130), (3780, 120)]
    spring = 470
    for (xa, wa), (xb, wb) in zip(piers, piers[1:]):
        if xa == 1340:
            continue
        bc, half = (xa + xb) / 2, (xb - xa) / 2 - (wa + wb) / 4
        dist = abs(bc - cx) / (W / 2)
        heat = max(.25, .85 - dist * .7)
        b += mouldings(bc, floor, spring, half, 1.7, [(46, STONE[1]), (34, STONE[5]), (24, STONE[2]),
                                                     (12, mix(STONE[5], BRONZE_RAMP[2], heat * .6)),
                                                     (4, STONE[1])])
        niche = d.linear([(0, STONE[0]), (.55, STONE[2]), (1, STONE[1])], 0, apex(spring, half, 1.7), 0, floor)
        b += path(poly(arch(bc, floor, spring, half, 1.7)), niche)
        b += masonry(bc - half, apex(spring, half, 1.7), bc + half, floor, 30, 90, rng)
        b += path(poly(arch(bc, floor, spring, half, 1.7)), DARK, opacity='0.35')
        b += lancet(d, bc, 860, 520, half * .36, heat, rng)
        b += shaft_of_light(bc, 860, half * .3, bc + (cx - bc) * .25, floor + 140, half * .9, .03 * heat + .01)
        # Blind trefoil dado under the window.
        for i in range(-2, 3):
            ax = bc + i * half * .36
            b += path(poly(arch(ax, floor - 30, floor - 120, half * .15, 2)), STONE[4])
            b += path(poly(arch(ax, floor - 34, floor - 116, half * .12, 2)), STONE[0])
        b += path(rect(bc - half, floor - 140, half * 2, 12), STONE[5])
        b += path(rect(bc - half, floor - 128, half * 2, 4), STONE[0])
    for x, _ in piers[1:-1]:
        b += roundel(x, 150, 82 if abs(x - cx) > 900 else 70)
    # The great arch: deep orders receding to the glowing apse.
    half, k, gspring = 450, 1.35, 600
    b += mouldings(cx, floor, gspring, half, k, [(110, STONE[1]), (92, STONE[6]), (78, STONE[3]),
                                                (64, BRONZE_RAMP[2]), (54, STONE[2]), (40, STONE[7]),
                                                (28, STONE[3]), (16, BRONZE_RAMP[3]), (8, STONE[1])])
    b += dogtooth(cx, floor, gspring, half, k, 59, 6, BRONZE_RAMP[4])
    b += dogtooth(cx, floor, gspring, half, k, 22, 4, STONE[7], 2)
    apse = d.radial([(0, '#7A3A2C'), (.4, STONE[5]), (.8, STONE[3]), (1, STONE[1])], 600, cx, 560)
    b += path(poly(arch(cx, floor, gspring, half, k)), apse)
    for i in range(1, 7):
        s = 1 - i * .045
        shape = [(cx + (x - cx) * s, vy + (y - vy) * s) for x, y in arch(cx, floor, gspring, half, k)]
        tone = mix(STONE[2], STONE[6], i / 7) if i % 2 else mix(STONE[1], STONE[4], i / 7)
        b += path(poly(shape), tone)
    s = 1 - 7 * .045
    back = [(cx + (x - cx) * s, vy + (y - vy) * s) for x, y in arch(cx, floor, gspring, half, k)]
    b += path(poly(back), apse)
    bx0, bx1 = cx - half * s, cx + half * s
    b += masonry(bx0, 120, bx1, vy + (floor - vy) * s, 22, 70, rng, AMBER)
    by = vy + (floor - vy) * s
    b += glow(cx, 560, 520, 420, '#F08A3C', .05, 10)
    b += rose(d, cx, 352, 118, 1, 16)
    for lx, top_spring, hh in ((cx - 150, 640, 50), (cx, 610, 58), (cx + 150, 640, 50)):
        b += lancet(d, lx, by - 40, top_spring, hh, .95, rng, lights=1)
    # Altar steps rising into the apse.
    for i in range(5):
        t0, t1 = i / 5, (i + 1) / 5
        y0, y1 = floor - (floor - by) * t1, floor - (floor - by) * t0
        w0, w1 = half * (1 - .3 * t1), half * (1 - .3 * t0)
        b += path(poly([(cx - w0, y0), (cx + w0, y0), (cx + w1, y1), (cx - w1, y1)]), mix(STONE[2], STONE[5], .6 - i * .1))
        b += path(rect(cx - w0, y0, w0 * 2, 3), mix(STONE[5], AMBER, .35))
    for lx in (cx - 150, cx, cx + 150):
        b += shaft_of_light(lx, 760, 34, lx + (lx - cx) * .8, floor + 150, 120, .028)
    b += floor_tiles(d, 0, W, floor, H, cx, vy, rng, cx, 900)
    b += glow(cx, floor + 40, 760, 110, '#E2703A', .05, 8)
    for lx in (cx - 150, cx, cx + 150):
        b += glow(lx, floor + 70, 46, 120, '#F08A3C', .05, 5)
    for x, w in piers:
        dist = (x - cx) / (W / 2)
        light = -max(.15, 1 - abs(dist)) if dist > 0 else max(.15, 1 - abs(dist))
        b += pier(d, x, spring if w < 150 else gspring, floor, w, light)
    b += brazier(d, cx - 300, by + 6, 46, rng) + brazier(d, cx + 300, by + 6, 46, rng)
    b += embers((cx - 520, 300, cx + 520, 900), 34, rng, 2)
    # Vault gloom overhead and falloff to the ends of the nave.
    b += gloom(W, 300, .5)
    b += vignette(W, H, cx, 650, 1500, 760, DARK, .09, 9, 1.55)
    return d.svg(), b


def library_capsule():
    """600x900. Fits author.py: emblem 256 px at (172, 100), wordmark x2 at y 398,
    name at y 620. The emblem sits in a carved recess under the great arch; the
    lettering falls in the arch's shadowed lower half."""
    W, H = 600, 900
    d, rng = Defs('lc'), random.Random(600)
    cx, floor = 300, 790
    wall = d.radial([(0, STONE[6]), (.35, STONE[3]), (1, STONE[0])], 520, cx, 228)
    b = path(rect(0, 0, W, H), wall) + masonry(0, -4, W, floor, 18, 54, rng)
    half, k, spring = 236, 1.3, 330
    b += mouldings(cx, floor, spring, half, k, [(52, STONE[1]), (42, STONE[6]), (34, STONE[3]),
                                               (26, BRONZE_RAMP[2]), (20, STONE[2]), (12, STONE[7]),
                                               (6, STONE[1])])
    b += dogtooth(cx, floor, spring, half, k, 23, 3, BRONZE_RAMP[4])
    inner = d.radial([(0, '#6E3A2C'), (.35, STONE[4]), (.75, STONE[2]), (1, STONE[0])], 420, cx, 228)
    b += path(poly(arch(cx, floor, spring, half, k)), inner)
    b += masonry(cx - half, 20, cx + half, floor, 16, 46, rng, AMBER)
    b += glow(cx, 228, 230, 230, '#F08A3C', .06, 10)
    # Flanking lancets low in the bay, their light raking the floor.
    for side in (-1, 1):
        lx = cx + side * 196
        b += lancet(d, lx, 640, 450, 24, .9, rng, lights=1)
        b += shaft_of_light(lx, 600, 18, lx - side * 40, floor + 60, 70, .035)
    b += octagon_recess(cx, 228, 256)
    # Shadowed band behind the lettering: a blind arcade in deep shade.
    for i in range(-3, 4):
        ax = cx + i * 62
        b += path(poly(arch(ax, 700, 610, 24, 2)), STONE[3])
        b += path(poly(arch(ax, 700, 614, 20, 2)), STONE[0])
    b += path(rect(cx - half, 700, half * 2, 8), STONE[5]) + path(rect(cx - half, 708, half * 2, 3), STONE[0])
    b += glow(cx, 520, 250, 200, DARK, .07, 9)
    b += glow(cx, 640, 230, 60, DARK, .1, 8)
    for i in range(4):
        y = floor - 22 * (i + 1)
        w = half - 30 + i * 0
        b += path(rect(cx - w + i * 18, y, (w - i * 18) * 2, 22), mix(STONE[2], STONE[4], i / 4))
        b += path(rect(cx - w + i * 18, y, (w - i * 18) * 2, 2), mix(STONE[5], AMBER, .25))
    b += floor_tiles(d, 0, W, floor, H, cx, 300, rng, cx, 300)
    b += pier(d, 18, spring, floor, 92, .55) + pier(d, 582, spring, floor, 92, -.55)
    b += brazier(d, 84, floor + 70, 34, rng) + brazier(d, 516, floor + 70, 34, rng)
    b += embers((60, 80, 540, 760), 26, rng, 1)
    b += gloom(W, 160, .5)
    b += vignette(W, H, cx, 420, 300, 470, DARK, .08, 8, 1.4)
    return d.svg(), b


def wide_capsule():
    """920x430. Fits author.py: emblem 230 px at (64, 90), wordmark x1.3 at (320, 130),
    name x0.8 at (320, 268). Emblem recess and lettering on a shaded wall bay; the
    glowing apse opens on the right."""
    W, H = 920, 430
    d, rng = Defs('wc'), random.Random(920)
    floor = 400
    wall = d.radial([(0, STONE[5]), (.4, STONE[3]), (1, STONE[0])], 700, 760, 200)
    b = path(rect(0, 0, W, H), wall) + masonry(0, -4, W, floor, 16, 50, rng)
    # Right: an arch into the apse with a lancet pair and the rose.
    acx, half, k, spring = 762, 118, 1.6, 220
    b += mouldings(acx, floor, spring, half, k, [(34, STONE[1]), (27, STONE[6]), (21, STONE[3]),
                                                (15, BRONZE_RAMP[2]), (10, STONE[2]), (5, STONE[7])])
    b += dogtooth(acx, floor, spring, half, k, 17.5, 2.6, BRONZE_RAMP[4])
    apse = d.radial([(0, '#7A3A2C'), (.5, STONE[4]), (1, STONE[1])], 240, acx, 200)
    b += path(poly(arch(acx, floor, spring, half, k)), apse)
    b += masonry(acx - half, 20, acx + half, floor, 12, 34, rng, AMBER)
    b += glow(acx, 210, 200, 190, '#F08A3C', .06, 10)
    b += rose(d, acx, 128, 46, 1)
    for lx in (acx - 46, acx + 46):
        b += lancet(d, lx, 330, 236, 22, .95, rng, lights=1)
        b += shaft_of_light(lx, 300, 14, lx + (lx - acx) * .6, floor + 30, 50, .035)
    for i in range(3):
        y = floor - 12 * (i + 1)
        w = half - i * 14
        b += path(rect(acx - w, y, w * 2, 12), mix(STONE[2], STONE[5], .5 - i * .1))
        b += path(rect(acx - w, y, w * 2, 2), mix(STONE[5], AMBER, .3))
    # Left: a wide shallow blind arch framing the lock-up, kept in shade.
    b += mouldings(318, floor, 190, 268, 1.25, [(18, STONE[1]), (12, STONE[5]), (6, STONE[2])])
    b += path(poly(arch(318, floor, 190, 268, 1.25)), d.linear([(0, STONE[2]), (1, STONE[0])], 0, 0, 0, floor))
    b += masonry(50, 0, 586, floor, 14, 44, rng)
    b += octagon_recess(179, 205, 230)
    b += glow(179, 205, 150, 150, '#F08A3C', .03, 8)
    b += glow(450, 215, 190, 120, DARK, .08, 9)
    b += floor_tiles(d, 0, W, floor, H, 460, 200, rng, acx, 260)
    b += pier(d, 616, spring, floor, 50, -.8) + pier(d, 912, spring, floor, 50, -.6)
    b += pier(d, 8, 190, floor, 40, .25)
    b += brazier(d, 880, floor + 22, 26, rng)
    b += embers((640, 60, 900, 380), 18, rng, .8)
    b += gloom(W, 110, .45)
    b += vignette(W, H, 470, 215, 470, 230, DARK, .07, 8, 1.35)
    return d.svg(), b
