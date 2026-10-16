"""Author original vector geometry and outline the bundled OFL Barlow face.
Design time only: fonttools==4.60.1. Exporting committed SVGs needs only resvg.
"""
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
BONE, BRONZE, EMBER, DARK = '#F8EAD5', '#B88960', '#B8583F', '#120C0D'
def path(d, fill): return f'<path fill="{fill}" d="{d}"/>'
def group(body, x=0, y=0, scale=1): return f'<g transform="translate({x} {y}) scale({scale})">{body}</g>'
def svg(w, h, body): return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{body}</svg>\n'
def save(name, w, h, body): (OUT/name).write_text(svg(w,h,body))
# Closed contours: an unbroken bronze threshold protects the ember. No strokes,
# fonts, filters or raster payloads; broad negative space survives small sizes.
mark = path('M128 8 222 78 222 214 128 248 34 214 34 78Z', DARK)
mark += path('M128 16 214 82 214 208 128 240 42 208 42 82Z', BRONZE)
mark += path('M128 29 201 87 201 199 128 226 55 199 55 87Z', '#52272B')
mark += path('M128 43 186 92 186 193 128 213 70 193 70 92Z', DARK)
mark += path('M128 43 186 92 179 103 128 62 77 103 70 92Z', BONE)
mark += path('M70 100 83 110 83 184 116 196 116 209 70 193Z M186 100 173 110 173 184 140 196 140 209 186 193Z', BRONZE)
mark += path('M128 87 140 131 158 150 128 184 98 150 116 131Z', EMBER)
mark += path('M128 118 140 150 128 167 116 150Z', BONE)
# Architectural feet and engraving stay secondary to the gate silhouette.
mark += path('M46 89 51 93 51 195 46 193Z M210 89 205 93 205 195 210 193Z M83 213 128 229 173 213 128 234Z', '#80533B')
mark += path('M128 19 133 30 128 35 123 30Z M46 204 53 204 53 211 46 211Z M203 204 210 204 210 211 203 211Z', BONE)
# Original upright slab lettering, drawn by hand; no franchise-derived face.
ddl = path('M0 0H28L42 12V52L28 64H0V54H6V10H0Z M18 12V52H25L30 47V17L25 12Z', BONE).replace('fill=', 'fill-rule="evenodd" fill=')
ddl += group(path('M0 0H28L42 12V52L28 64H0V54H6V10H0Z M18 12V52H25L30 47V17L25 12Z', BONE).replace('fill=', 'fill-rule="evenodd" fill='), 52)
ddl += path('M104 0H130V10H123V52H145V44H157V64H104V54H110V10H104Z', BONE)
ddl += path('M0 72H68L78 78 88 72H157V75H89L78 84 67 75H0Z', EMBER)
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
save('horizontal.svg',440,128,group(mark,0,0,.5)+group(ddl,152,22,1))
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
save('eviternity-cover.svg',512,586,group(scene(True),-134,0,.6512))
save('fallback-cover.svg',512,586,group(scene(),-134,0,.6512))
# Art band has an opaque feathered entry represented by narrow vector strips.
for title,ivory in [('eviternity',True),('fallback',False)]:
    body=group(scene(ivory),0,-85,1/3)
    for x in range(144):
        t=x/144; a=1-t*t*(3-2*t)
        body+=f'<path fill="{DARK}" fill-opacity="{a:.6f}" d="M{x} 0H{x+1}V88H{x}Z"/>'
    save(title+'-band.svg',400,88,body)
# Dedicated Steam compositions retain an open central hero safe area.
art=scene()
save('library-capsule.svg',600,900,group(art,-300,0)+group(mark,172,100,1)+group(ddl,143,398,2)+group(name,(600-nw)/2,620))
save('wide-capsule.svg',920,430,group(art,0,-240,.767)+path('M40 50H570V370H40Z',DARK)+group(mark,64,90,.9)+group(ddl,320,130,1.3)+group(name,320,268,.8))
save('hero.svg',3840,1240,group(art,0,-800,3.2))
save('steam-logo.svg',1280,720,group(mark,465,30,1.37)+group(ddl,443.75,404,2.5)+group(name,(1280-nw*2)/2,641,2))
save('social-preview.svg',1280,640,group(art,0,-245,1.067)+path('M70 90H1210V550H70Z',DARK)+group(mark,108,156,1.25)+group(ddl,500,180,2.8)+group(name,500,460,1.55))
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
