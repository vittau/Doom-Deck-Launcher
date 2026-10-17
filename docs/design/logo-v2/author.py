"""Author original filled-path concepts; reuse the existing OFL Barlow name outlines.
Python stdlib only. Does not modify the current production identity.
"""
from pathlib import Path
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
DARK, BONE, BRONZE, BLOOD, AMBER = '#120C0D', '#F8EAD5', '#B88960', '#52272B', '#FFD078'


def path(d, fill, even=False):
    return f'<path fill="{fill}"' + (' fill-rule="evenodd"' if even else '') + f' d="{d}"/>'


def group(body, x=0, y=0, scale=1):
    return f'<g transform="translate({x} {y}) scale({scale})">{body}</g>'


def svg(w, h, body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{body}</svg>\n'


# Newly drawn monoline slab DDL: broad open counters, upright sides, no blackletter.
d = path('M0 0H25L42 15V49L25 64H0Z M13 13V51H21L29 43V21L21 13Z', BONE, True)
ddl = d + group(d, 54) + path('M108 0H122V50H154V64H108Z', BONE)
# Reuse only the committed Barlow name paths, not the Ember Gate or DDL geometry.
root = ET.parse(HERE.parent / 'logo/full-name.svg').getroot()
name = ''.join(ET.tostring(child, encoding='unicode') for child in list(root)[1])
name = name.replace('ns0:', '').replace(':ns0', '')

# Cinder Key: a heavy pointed escutcheon with a single keyhole cut through it.
key = path('M128 4 232 64V180L128 252 24 180V64Z', DARK)
key += path('M128 16 220 72V174L128 238 36 174V72Z', BRONZE)
key += path('M128 34 204 82V166L128 218 52 166V82Z', BLOOD)
key += path('M128 50 180 88V122L150 148 160 190H96L106 148 76 122V88Z', DARK)
key += path('M128 67 163 95V116L140 137 145 173H111L116 137 93 116V95Z', BONE)
key += path('M128 87 140 105 128 122 116 105Z', AMBER)
key += path('M54 155 70 165 70 176 54 166Z M202 155 186 165 186 176 202 166Z', BRONZE)

# Rose Compass: a broad eight-sided rose window. Four separated petals frame an ember.
rose = path('M80 4H176L252 80V176L176 252H80L4 176V80Z', DARK)
rose += path('M86 18H170L238 86V170L170 238H86L18 170V86Z', BRONZE)
rose += path('M92 34H164L222 92V164L164 222H92L34 164V92Z', BLOOD)
rose += path('M128 42 157 94 128 110 99 94Z M214 128 162 157 146 128 162 99Z M128 214 99 162 128 146 157 162Z M42 128 94 99 110 128 94 157Z', DARK)
rose += path('M128 56 145 90 128 99 111 90Z M200 128 166 145 157 128 166 111Z M128 200 111 166 128 157 145 166Z M56 128 90 111 99 128 90 145Z', BONE)
rose += path('M128 111 145 128 128 145 111 128Z', AMBER)
rose += path('M69 58 80 69 69 80 58 69Z M187 58 198 69 187 80 176 69Z M69 176 80 187 69 198 58 187Z M187 176 198 187 187 198 176 187Z', BRONZE)

# Ember Folio: open carved map atlas, low/wide silhouette, central amber bookmark.
folio = path('M8 42 104 16 128 33 152 16 248 42V211L152 239 128 225 104 239 8 211Z', DARK)
folio += path('M20 52 101 30 128 48 155 30 236 52V201L155 225 128 211 101 225 20 201Z', BRONZE)
folio += path('M34 64 99 46 118 59V196L99 209 34 190Z M222 64 157 46 138 59V196L157 209 222 190Z', BLOOD)
folio += path('M48 76 96 63 104 69V105L80 94 48 104Z M208 76 160 63 152 69V105L176 94 208 104Z', BONE)
folio += path('M48 122 78 112 104 125V140L78 127 48 137Z M48 155 78 145 104 158V173L78 160 48 170Z M208 122 178 112 152 125V140L178 127 208 137Z M208 155 178 145 152 158V173L178 160 208 170Z', BRONZE)
folio += path('M120 64H136V186L128 199 120 186Z', AMBER)


def size_label(value, x, y):
    digits = {'2': ['111','001','111','100','111'],
              '3': ['111','001','111','001','111'],
              '4': ['101','101','111','001','001'],
              '5': ['111','100','111','001','111'],
              '6': ['111','100','111','101','111']}
    body = ''
    for i, digit in enumerate(str(value)):
        for row, bits in enumerate(digits[digit]):
            for column, bit in enumerate(bits):
                if bit == '1':
                    px, py = x + i * 16 + column * 4, y + row * 4
                    body += path(f'M{px} {py}h4v4h-4Z', BRONZE)
    return body


def main():
    for slug, mark in [('cinder-key', key), ('rose-compass', rose), ('ember-folio', folio)]:
        folder = HERE / slug
        folder.mkdir(exist_ok=True)
        lockup = mark + path('M284 24H752V232H284Z', DARK)
        lockup += path('M284 24H290V232H284Z', BRONZE)
        lockup += group(ddl, 316, 34, 2.2) + group(name, 316, 184)
        (folder / 'emblem.svg').write_text(svg(256, 256, mark))
        (folder / 'horizontal.svg').write_text(svg(768, 256, lockup))
        sheet = ''
        for row, bg in enumerate([DARK, BONE]):
            y = row * 640
            sheet += path(f'M0 {y}H1440V{y+640}H0Z', bg)
            for x, size in [(32, 32), (208, 64), (624, 256)]:
                sheet += size_label(size, x, y + 12)
                sheet += group(mark, x, y + 48, size / 256)
                sheet += group(lockup, x, y + 344, size / 256)
                # Geometric size ticks: 1, 2 or 8 blocks of 32 px, not font text.
                for tick in range(size // 32):
                    sheet += path(f'M{x+tick*16} {y+616}h8v4h-8Z', BRONZE)
        (folder / 'contact-sheet.svg').write_text(svg(1440, 1280, sheet))


if __name__ == '__main__':
    main()
