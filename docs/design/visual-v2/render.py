#!/usr/bin/env python3
"""Offline, deterministic 1280×800 concept compositor. Requires Pillow 12.3.0.
Run: python render.py [ember-cathedral|bastion-terminal|cinder-1993]
Generated atlas sources are immutable; tile boxes explicitly match their seams.
"""
from pathlib import Path
import sys, json
from PIL import Image, ImageDraw, ImageFont, ImageOps
ROOT=Path(__file__).resolve().parent
F=ROOT/'fonts'
STYLES={
 'ember-cathedral':dict(name='Ember Cathedral',tag='CARVED STONE / OXBLOOD / BONE',canvas='#120C0D',panel='#211719',raised='#352527',text='#F8EAD5',muted='#D0BCA7',focus='#FFD078',accent='#B8583F',edge='#80533B',display='pirataone-PirataOne-Regular.ttf',body='barlow-Barlow-Regular.ttf',seam=438),
 'bastion-terminal':dict(name='Bastion Terminal',tag='OLIVE ENAMEL / BRASS / SIGNAL RED',canvas='#10130F',panel='#1D241D',raised='#303A2C',text='#F4F1DA',muted='#C6CBB5',focus='#FFD078',accent='#C36446',edge='#74754B',display='russoone-RussoOne-Regular.ttf',body='InterVariable.ttf',seam=512),
 'cinder-1993':dict(name='Cinder 1993',tag='UMBER STONE / COPPER / BEVELLED RED',canvas='#17100B',panel='#2A2016',raised='#403221',text='#FFF0CC',muted='#D5C3A1',focus='#FFD078',accent='#E18A63',edge='#98734C',display='teko-Teko[wght].ttf',body='barlow-Barlow-Regular.ttf',seam=512),
}
def lum(c):
 a=[int(c[i:i+2],16)/255 for i in (1,3,5)]
 a=[x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4 for x in a]
 return sum(x*y for x,y in zip(a,(.2126,.7152,.0722)))
def ratio(a,b):
 x,y=sorted([lum(a),lum(b)])
 return (y+.05)/(x+.05)
class Render:
 def __init__(self,slug):
  self.slug=slug;self.s=STYLES[slug]; self.out=ROOT/slug
  atlas=Image.open(self.out/'art-atlas.png').convert('RGB'); w,h=atlas.size;y=self.s['seam']
  self.tiles=[atlas.crop((i*w//3,j0,(i+1)*w//3,j1)) for j0,j1 in [(0,y),(y,h)] for i in range(3)]
 def font(self,size,display=False):return ImageFont.truetype(str(F/self.s['display' if display else 'body']),size)
 def rect(self,box,fill,outline=None,width=1):self.d.rectangle(box,fill=fill,outline=outline,width=width)
 def text(self,x,y,t,size=22,color=None,display=False):
  self.d.text((x,y),t,font=self.font(size,display),fill=color or self.s['text'],stroke_width=0)
 def fit(self,x,y,t,maxwidth,size=22,color=None,display=False):
  while self.d.textlength(t,font=self.font(size,display))>maxwidth and size>20:size-=1
  assert self.d.textlength(t,font=self.font(size,display))<=maxwidth,(t,maxwidth)
  self.text(x,y,t,size,color,display)
 def art(self,idx,box):self.im.paste(ImageOps.fit(self.tiles[idx],(box[2]-box[0],box[3]-box[1])),box[:2])
 def panel(self,box):
  # Generated carved material along perimeter; opaque center protects all body text.
  self.art(1,box);x,y,r,b=box
  self.rect((x+9,y+9,r-9,b-9),self.s['panel'])
  self.rect((x+11,y+11,r-11,b-11),None,self.s['edge'])
  if self.slug=='bastion-terminal':
   for xx,yy in [(x+5,y+5),(r-5,y+5),(x+5,b-5),(r-5,b-5)]:self.d.ellipse((xx-2,yy-2,xx+2,yy+2),fill=self.s['focus'])
 def button(self,box,label,primary=False,focus=False):
  x,y,r,b=box
  if focus:
   self.rect((x-8,y-8,r+8,b+8),self.s['canvas'],self.s['focus'],4)
  self.rect(box,self.s['focus'] if primary else self.s['raised'],self.s['edge'],1)
  self.d.line((x+2,y+2,r-2,y+2),fill=self.s['text'] if primary else self.s['edge'],width=1)
  self.d.line((x,b-1,r,b-1),fill=self.s['canvas'],width=3)
  self.fit(x+16,y+13,label,r-x-32,22,self.s['canvas'] if primary else self.s['text'])
 def row(self,box,title,detail):
  self.rect(box,self.s['raised']);x,y,r,b=box
  self.fit(x+16,y+9,title,r-x-32,22)
  self.fit(x+16,y+39,detail,r-x-32,20,self.s['muted'])
 def base(self,title,subtitle,setup=False):
  self.im=Image.new('RGB',(1280,800),self.s['canvas']);self.d=ImageDraw.Draw(self.im)
  # Wall atmosphere remains visible in gutters and above/below panels only.
  self.art(0,(0,104,1280,736))
  overlay=Image.new('RGB',(1280,632),self.s['canvas'])
  region=self.im.crop((0,104,1280,736));self.im.paste(Image.blend(region,overlay,.72),(0,104))
  self.rect((0,0,1280,104),self.s['canvas']);self.rect((0,736,1280,800),self.s['canvas'])
  self.art(2,(40,24,80,80));self.text(88,27,'DDL',32,display=True)
  for i,t in enumerate(['Home','Discover','Library','Settings']):
   x=214+i*180
   if i==0:self.rect((x,24,x+148,76),self.s['raised']);self.rect((x+24,80,x+124,83),self.s['focus'])
   self.text(x+25,36,t,22)
  self.text(1020,38,'DoomDeckLauncher',20,self.s['muted'])
  self.rect((40,120,1240,210),self.s['canvas'])
  if self.slug=='cinder-1993':self.text(42,127,title,42,'#723824',True)
  self.text(40,122,title,40,self.s['text'] if self.slug!='cinder-1993' else self.s['accent'],True)
  self.text(40,177,subtitle,22,self.s['muted'])
  self.d.line((40,103,1240,103),fill=self.s['edge'])
 def footer(self,setup=False):
  items=[('A','Play' if setup else 'Continue'),('B','Back')]
  # X/Y are omitted when no corresponding production action is available.
  if not setup:items += [('L1',''),('R1','Tabs')]
  items += [('Start','Menu')]
  x=40
  for badge,label in items:
   w=28 if len(badge)==1 else (40 if badge!='Start' else 58)
   box=(x,754,x+w,782)
   if len(badge)==1:self.d.ellipse(box,fill=self.s['raised'],outline=self.s['muted'])
   else:self.d.rounded_rectangle(box,radius=4,fill=self.s['raised'],outline=self.s['muted'])
   tw=self.d.textlength(badge,font=self.font(18));self.text(x+(w-tw)/2,757,badge,18)
   if label:self.text(x+w+8,755,label,20,self.s['muted'])
   x+=w+ (self.d.textlength(label,font=self.font(20))+32 if label else 8)
 def home(self):
  self.base('Home','Your last profile, ready again.')
  self.panel((40,230,824,708));self.panel((856,230,1240,708))
  self.art(3,(64,254,304,554));self.rect((72,520,296,548),self.s['canvas']);self.text(78,523,'Original concept art',18,self.s['muted'])
  self.text(336,274,'SIGIL',48,display=True)
  self.text(336,343,'John Romero',22,self.s['muted']);self.text(336,380,'Ultimate DOOM · Default profile',22)
  self.button((336,470,784,526),'Continue',True,True)
  self.text(880,262,'Ready to play',29,display=True)
  for y,t in [(326,'DSDA-Doom 0.30.1'),(364,'Ultimate DOOM found'),(402,'DOOM II found')]:self.text(880,y,t,22)
  self.text(880,452,'Not yet verified',22,self.s['focus'])
  self.button((880,538,1216,594),'Discover mapsets')
  # Same production hero panel; static catalog context uses its spare lower space.
  self.d.line((64,577,800,577),fill=self.s['edge'])
  self.text(64,588,'Also in your library',20,self.s['muted'])
  self.art(4,(64,624,112,682));self.text(124,622,'SIGIL II',26,display=True);self.text(124,657,'John Romero · Ultimate DOOM',20,self.s['muted'])
  self.art(5,(442,624,490,682));self.fit(502,622,'AUGER;ZENITH',292,26,display=True);self.text(502,657,'DBP 37 · DOOM II',20,self.s['muted'])
  self.footer();self.save('home.png')
 def setup(self):
  self.base('Play setup / AUGER;ZENITH','Doomer Boards Project 37 · DOOM II',True)
  self.panel((40,230,720,708));self.panel((744,230,1240,708))
  self.row((64,254,696,334),'Default profile','Changes are saved to this mapset’s profile.')
  self.row((64,350,696,430),'AUGER;ZENITH · DBP37_AUGZEN.wad','Required by the recipe · cannot be unchecked')
  self.row((64,446,696,526),'Exclusive add-on · choose one','')
  for x,r,t,selected in [(80,178,'None',True),(190,420,'Alternate Blaster',False),(432,680,'Deckard Blaster',False)]:
   self.rect((x,488,r,521),self.s['canvas'] if selected else self.s['raised'],self.s['focus'] if selected else self.s['edge'])
   self.fit(x+8,492,t,r-x-16,20)
   if selected:self.rect((x+2,490,x+4,519),self.s['focus'])
  self.row((64,542,696,622),'Advanced','Compatibility replacements require confirmation.')
  self.button((64,638,368,694),'Skill 3');self.button((384,638,696,694),'Pistol start: Off')
  self.text(768,246,'Will launch',30,display=True)
  self.text(768,299,'DOOM II · base game found',22)
  self.text(768,348,'1 content file · recipe order locked',22,self.s['muted'])
  self.text(768,393,'1  DBP37_AUGZEN.wad',22)
  self.text(768,440,'Add-on: None',22,self.s['muted'])
  self.text(768,481,'Complevel 2 · recipe recommendation',20)
  self.text(768,508,'Not yet verified',20,self.s['focus'])
  self.text(768,542,'DSDA-Doom 0.30.1',20)
  self.text(768,580,'Skill 3 · Pistol start: Off',20)
  self.button((768,620,1216,676),'Play',True,True)
  self.footer(True);self.save('play-setup-auger.png')
 def sheet(self):
  self.base(self.s['name'],'VISUAL SYSTEM / 1280 × 800 / ORIGINAL ART')
  self.panel((40,230,660,708));self.panel((684,230,1240,708))
  self.text(64,249,'Palette & typography',30,display=True)
  for i,k in enumerate(['canvas','panel','raised','text','muted','focus']):
   x=64+i*96;self.rect((x,300,x+80,350),self.s[k]);self.text(x,359,k,16,self.s['muted']);self.text(x,382,self.s[k],16)
  self.text(64,425,'AUGER;ZENITH',44,display=True)
  self.text(64,481,'Display / '+{'ember-cathedral':'Pirata One','bastion-terminal':'Russo One','cinder-1993':'Teko'}[self.slug],20,self.s['muted'])
  self.text(64,519,'Body / '+('Inter 4.1' if self.slug=='bastion-terminal' else 'Barlow Regular')+' / 22 px',22)
  self.text(64,556,'DBP37_AUGZEN.wad · exact filename case',22)
  self.text(64,600,'Main text / panel: %.2f:1'%ratio(self.s['text'],self.s['panel']),22)
  self.text(64,637,'Muted / raised: %.2f:1'%ratio(self.s['muted'],self.s['raised']),22)
  self.text(708,249,'Materials & controls',30,display=True)
  for i in range(3):self.art(i,(708+i*168,300,860+i*168,420))
  self.text(708,432,'Wall / carved frame / original emblem',20,self.s['muted'])
  self.button((716,484,944,540),'Default');self.button((976,484,1200,540),'Focused',True,True)
  self.row((708,568,1216,640),'Unavailable','Explicit reason · no low-opacity text')
  x=708
  for badge in ['A','B','X','Y']:
   self.d.ellipse((x,659,x+28,687),fill=self.s['raised'],outline=self.s['muted']);self.text(x+8,661,badge,18);x+=44
  for badge in ['L1','R1','Start']:
   w=58 if badge=='Start' else 40;self.d.rounded_rectangle((x,659,x+w,687),4,fill=self.s['raised'],outline=self.s['muted']);self.text(x+6,661,badge,18);x+=w+12
  self.text(40,755,self.s['tag'],20,self.s['muted']);self.save('style-sheet.png')
 def save(self,name):
  self.im.save(self.out/name,optimize=True)
  assert (self.out/name).stat().st_size<2_000_000
 def run(self):self.home();self.setup();self.sheet()
if __name__=='__main__':
 for slug in sys.argv[1:] or STYLES:
  Render(slug).run()
