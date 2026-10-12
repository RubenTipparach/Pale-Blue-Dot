"""Normalize committed image-generation outputs. Requires Pillow; never calls AI.

Run from any directory. The exact prompts, palettes and conversion contract are
in swatches/manifest.json. Author.py only needs the resulting small PNG files.
"""
import json
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent


def normalize(entry):
    source = Image.open(ROOT/'swatches'/entry['generated']).convert('RGBA')
    size = entry['size']
    colors = [tuple(bytes.fromhex(h)) for h in entry['palette']]
    background = tuple(bytes.fromhex(entry['alpha_background']))
    result = Image.new('RGBA',tuple(size))
    for y in range(size[1]):
        for x in range(size[0]):
            sx = min(source.width-1,int((x+.5)*source.width/size[0]))
            sy = min(source.height-1,int((y+.5)*source.height/size[1]))
            r,g,b,a = source.getpixel((sx,sy))
            rgb = tuple((c*a+background[i]*(255-a)+127)//255 for i,c in enumerate((r,g,b)))
            chosen = min(colors,key=lambda c:sum((c[i]-rgb[i])**2 for i in range(3)))
            result.putpixel((x,y),chosen+(255,))
    for y in range(size[1]): result.putpixel((size[0]-1,y),result.getpixel((0,y)))
    for x in range(size[0]): result.putpixel((x,size[1]-1),result.getpixel((x,0)))
    result.save(ROOT/'swatches'/entry['normalized'])
    return result


def main():
    manifest = json.loads((ROOT/'swatches'/'manifest.json').read_text())
    entries = manifest['swatches']
    contact = Image.new('RGB',(700,((len(entries)+3)//4)*190),'#182731')
    draw = ImageDraw.Draw(contact)
    for i,entry in enumerate(entries):
        tile = normalize(entry)
        repeated = Image.new('RGBA',(96,96))
        for y in range(3):
            for x in range(3): repeated.paste(tile,(x*32,y*32))
        # The centre crop puts wrapped boundaries in the middle of each preview.
        preview = repeated.crop((16,16,80,80)).resize((160,160),Image.Resampling.NEAREST)
        x,y = (i%4)*175,(i//4)*190
        contact.paste(preview,(x,y+20));draw.text((x,y),entry['name'],fill='white')
        print(entry['name'],len(tile.getcolors()),'colors',tile.size)
    output = ROOT.parents[2]/'output'/'vehicles-improve'/'swatch-wraps.png'
    output.parent.mkdir(parents=True,exist_ok=True)
    contact.save(output)


if __name__ == '__main__': main()
