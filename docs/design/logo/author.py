"""Author original vector geometry and outline the bundled OFL Barlow face.
Design time only: fonttools==4.60.1. Exporting committed SVGs needs only resvg.
"""
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
BLOOD, AMBER = '#52272B', '#FFD078'
BONE, BRONZE, EMBER, DARK = '#F8EAD5', '#B88960', '#B8583F', '#120C0D'
def path(d, fill, even=False):
    return f'<path fill="{fill}"' + (' fill-rule="evenodd"' if even else '') + f' d="{d}"/>'
def group(body, x=0, y=0, scale=1): return f'<g transform="translate({x} {y}) scale({scale})">{body}</g>'
def svg(w, h, body): return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{body}</svg>\n'
def save(name, w, h, body): (OUT/name).write_text(svg(w,h,body))
# Rose Compass production geometry, selected in D-023. Same filled contours as
# the retained phase-A concept; scenery below stays unchanged.
mark = path('M80 4H176L252 80V176L176 252H80L4 176V80Z', DARK)
mark += path('M86 18H170L238 86V170L170 238H86L18 170V86Z', BRONZE)
mark += path('M92 34H164L222 92V164L164 222H92L34 164V92Z', BLOOD)
mark += path('M128 42 157 94 128 110 99 94Z M214 128 162 157 146 128 162 99Z M128 214 99 162 128 146 157 162Z M42 128 94 99 110 128 94 157Z', DARK)
mark += path('M128 56 145 90 128 99 111 90Z M200 128 166 145 157 128 166 111Z M128 200 111 166 128 157 145 166Z M56 128 90 111 99 128 90 145Z', BONE)
mark += path('M128 111 145 128 128 145 111 128Z', AMBER)
mark += path('M69 58 80 69 69 80 58 69Z M187 58 198 69 187 80 176 69Z M69 176 80 187 69 198 58 187Z M187 176 198 187 187 198 176 187Z', BRONZE)
# Original upright wordmark from the selected concept.
d = path('M0 0H25L42 15V49L25 64H0Z M13 13V51H21L29 43V21L21 13Z', BONE, True)
ddl = d + group(d, 54) + path('M108 0H122V50H154V64H108Z', BONE)
font=TTFont(ROOT/'crates/ddl/ui/assets/Barlow-Regular.ttf'); glyphs=font.getGlyphSet(); cmap=font.getBestCmap()
def lettering(text, size, color=BONE):
    x=0; body=''; scale=size/font['head'].unitsPerEm
    for c in text:
        g=glyphs[cmap[ord(c)]]; pen=SVGPathPen(glyphs); g.draw(pen)
        body += f'<g transform="translate({x:.4f} {size}) scale({scale:.6f} {-scale:.6f})">'+path(pen.getCommands(),color)+'</g>'
        x+=g.width*scale
    return body,x
name,nw=lettering('DoomDeckLauncher',38)
save('emblem.svg',256,256,mark)
save('horizontal.svg',330,128,group(mark,0,0,.5)+group(ddl,152,22,1))
save('stacked.svg',480,400,group(mark,112,0,1)+group(ddl,161.5,269,1)+group(name,(480-nw)/2,354))
save('full-name.svg',700,160,group(mark,0,0,.625)+group(name,184,54,1.45))
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
# Dedicated Steam compositions retain an open central hero safe area.
art=scene()
save('library-capsule.svg',600,900,group(art,-300,0)+path('M90 370H510V680H90Z',DARK)+group(mark,172,100,1)+group(ddl,143,398,2)+group(name,(600-nw)/2,620))
save('wide-capsule.svg',920,430,group(art,0,-240,.767)+path('M40 50H650V370H40Z',DARK)+group(mark,64,90,.9)+group(ddl,320,130,1.3)+group(name,320,268,.8))
save('hero.svg',3840,1240,group(art,0,-800,3.2))
save('steam-logo.svg',1280,720,group(mark,465,30,1.37)+group(ddl,443.75,404,2.5)+group(name,(1280-nw*2)/2,618,2))
# Social scenes have their own original raster layers and outlined copy.
# Rebuild after the production masters above, never restore the old scenery.
import runpy
runpy.run_path(str(OUT/'author-social.py'), run_name='__main__')
# Contact sheet is itself vector, including labels and dark/light backgrounds.
body=''
for row,bg in enumerate([DARK,'#F8EAD5']):
    body+=path(f'M0 {row*360}H900V{row*360+360}H0Z',bg)
    for col,size in enumerate([32,64,256]):
        x=col*300+(300-size)/2; y=row*360+56
        body+=group(mark,x,y,size/256)
        label,lw=lettering(f'{size} px',22, BONE if row==0 else DARK)
        body+=group(label,col*300+(300-lw)/2,row*360+322)
save('contact-sheet.svg',900,720,body)
# Exact same masters compiled into Slint, with byte equality verified by export.
for name in ['emblem','horizontal','stacked','full-name']:
    (ROOT/f'crates/ddl/ui/assets/logo/{name}.svg').write_bytes((OUT/f'{name}.svg').read_bytes())
