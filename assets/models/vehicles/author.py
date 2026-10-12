"""Run inside Blender 4.4. Inputs come from `cargo run --example vehicle_art`.

This script owns only its PBD_AUTHOR scene. Coordinates supplied to helpers are
game metres (+X right, +Y up, +Z aft); export recovers those same coordinates.
"""
import bpy
import bmesh
import json
import math
import struct
import zlib
from pathlib import Path
from mathutils import Matrix, Vector, Euler

ROOT = Path(__file__).resolve().parent
DENSITY = 16
PADDING = 2
C = Matrix(((1, 0, 0), (0, 0, -1), (0, 1, 0)))
RAMPS = {
    'white': ('A8B7B5','CFD8D2','E5E9E2','F6F4EA'),
    'coral': ('A74438','D96549','EA8763','F5AE83'),
    'navy': ('182731','263A49','405965','71888C'),
    'metal': ('405965','71888C','A8B7B5','CFD8D2'),
    'teal': ('2C6963','367C77','418880','609B8D'),
    'pale': ('A8B7B5','CFD8D2','E5E9E2','F6F4EA'),
    'wood': ('79523A','AE7C50','D4A875','EACA94'),
    'amber': ('AE7C50','D4A875','EACA94','F6F4EA'),
}


def rgba(h):
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4)) + (255,)


def reset(kind):
    global SCENE, DATA, KIND, TAGS, SWATCHES, USED_SWATCHES
    DATA = json.loads((ROOT / 'inputs.json').read_text())
    KIND, TAGS = kind, {}
    SWATCHES, USED_SWATCHES = {}, set()
    old = bpy.data.scenes.get('PBD_AUTHOR')
    SCENE = bpy.data.scenes.new('PBD_AUTHOR_NEW')
    bpy.context.window.scene = SCENE
    if old:
        for obj in list(old.objects):
            bpy.data.objects.remove(obj, do_unlink=True)
        bpy.data.scenes.remove(old)
    SCENE.name = 'PBD_AUTHOR'
    SCENE.unit_settings.system = 'METRIC'
    SCENE.unit_settings.scale_length = 1.0
    return DATA['specs'][kind]


def node(name, parent=None, at=(0, 0, 0), basis=None, data=None):
    obj = bpy.data.objects.new(name, data)
    SCENE.collection.objects.link(obj)
    obj.parent = parent
    obj.location = C @ Vector(at)
    if basis is not None:
        obj.rotation_mode = 'QUATERNION'
        obj.rotation_quaternion = (C @ basis @ C.inverted()).to_quaternion()
    return obj


def poly(name, vertices, faces, tag, parent=None, at=(0, 0, 0), basis=None, closed=True):
    vertices = [Vector(p) for p in vertices]
    clean = []
    for face in faces:
        if len(face) < 3:
            continue
        a, b, c = (vertices[i] for i in face[:3])
        normal = (b-a).cross(c-a)
        if normal.length < 1e-10:
            continue
        normal.normalize()
        if any(abs((vertices[i]-a).dot(normal)) > 1e-6 for i in face[3:]):
            clean.extend((face[0], face[j], face[j+1]) for j in range(1, len(face)-1))
        else:
            clean.append(face)
    mesh = bpy.data.meshes.new(name + '_mesh')
    mesh.from_pydata([C @ p for p in vertices], [], clean)
    mesh.update()
    if closed:
        bm = bmesh.new()
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        bm.to_mesh(mesh)
        bm.free()
    obj = node(name, parent, at, basis, mesh)
    TAGS[name] = tag
    return obj


def box(name, size, at, tag, parent=None):
    x, y, z = (v/2 for v in size)
    return poly(name, [(-x,-y,-z),(x,-y,-z),(x,y,-z),(-x,y,-z),
                       (-x,-y,z),(x,-y,z),(x,y,z),(-x,y,z)],
                [(0,3,2,1),(4,5,6,7),(0,1,5,4),(3,7,6,2),(0,4,7,3),(1,2,6,5)],
                tag, parent, at)


def rings(name, sections, tag, parent=None, at=(0,0,0), axis='Z', sides=10):
    # Sections: axial position, radius along first cross axis, second radius.
    points = []
    for section in sections:
        t, a, b = section[:3]
        offset = section[3] if len(section) > 3 else 0
        for j in range(sides):
            u = 2*math.pi*j/sides
            p = (a*math.cos(u), b*math.sin(u)+offset, t)
            if axis == 'Y': p = (p[0], p[2], p[1])
            if axis == 'X': p = (p[2], p[0], p[1])
            points.append(p)
    faces = [tuple(reversed(range(sides)))]
    for k in range(len(sections)-1):
        for j in range(sides):
            q = (j+1) % sides
            faces.append((k*sides+j, k*sides+q, (k+1)*sides+q, (k+1)*sides+j))
    faces.append(tuple((len(sections)-1)*sides+j for j in range(sides)))
    return poly(name, points, faces, tag, parent, at)


def prism(name, outline, thickness, tag, parent=None, at=(0,0,0)):
    # Outline in local X/Z. A solid, flat-normal extrusion along local Y.
    n = len(outline)
    points = [(x, y, z) for y in (-thickness/2, thickness/2) for x,z in outline]
    faces = [tuple(reversed(range(n))), tuple(n+j for j in range(n))]
    faces += [(j,(j+1)%n,(j+1)%n+n,j+n) for j in range(n)]
    return poly(name, points, faces, tag, parent, at)


def foil_frame(spec):
    chord, normal = Vector(spec['chord']), Vector(spec['normal'])
    return Matrix((chord.cross(normal), normal, -chord)).transposed()


def airfoil(name, spec, span, chord, control, tag='white'):
    frame = node(name, at=spec['at'], basis=foil_frame(spec))
    # Faceted leading edge and a genuinely separate trailing control strip.
    profile = [(-chord/2,0),(-chord*.40,.075),(chord/4,.035),
               (chord/4,-.035),(-chord*.40,-.045)]
    pts = [(x,y,z) for x in (-span/2,span/2) for z,y in profile]
    n = len(profile)
    faces = [tuple(reversed(range(n))),tuple(n+i for i in range(n))]
    faces += [(j,(j+1)%n,(j+1)%n+n,j+n) for j in range(n)]
    poly(name+'_skin', pts, faces, tag, frame)
    hinge = node(control, frame, (0,0,chord/4))
    prism(control+'_skin', [(-span/2,0),(span/2,0),(span/2,chord/4),(-span/2,chord/4)],
          .06, 'coral', hinge)
    return frame


def crew(eye):
    parent = node('crew', at=eye)
    rings('crew_jacket', [(-.85,.23,.17),(-.35,.22,.18),(-.27,.14,.13)],
          'amber', parent, axis='Y', sides=8)
    rings('crew_helmet', [(-.25,.10,.10),(-.18,.16,.15),(.04,.14,.13),(.10,.05,.05)],
          'pale', parent, axis='Y', sides=8)
    box('crew_visor', (.21,.09,.025), (0,-.04,-.14), 'navy', parent)
    for side in (-1,1):
        rings('crew_arm_'+str(side), [(-.84,.07,.07),(-.40,.09,.09)],
              'navy', parent, (side*.27,0,-.08), axis='Y', sides=6)
        box('crew_leg_'+str(side), (.17,.18,.48), (side*.13,-.90,-.15), 'navy', parent)


def build_kestrel():
    s = reset('kestrel')
    rings('fuselage', [(-4.75,.025,.025),(-3.5,.66,.62),(-2.2,.75,.72),
                      (1.8,.68,.63),(3.8,.36,.38,.55),(5.25,.16,.21,.72)], 'white')
    rings('canopy', [(-3.15,.12,.10),(-2.6,.53,.47),(-1.65,.55,.43),(-1.05,.12,.08)],
          'navy', at=(0,.63,0), sides=8)
    box('nose_badge', (.31,.02,.50), (0,.64,-3.0), 'coral')
    box('cabin_hatch', (.025,.64,1.05), (.71,.03,-.20), 'coral')
    for side, spec in zip(('left','right'), DATA['wing_panels']):
        # panel order follows core; names are chosen from the physical X sign.
        side = 'right' if spec['at'][0] > 0 else 'left'
        span = math.sqrt(s['wing']['panel_area_m2']*s['wing']['aspect']*2)/2
        airfoil('wing_'+side, spec, span, spec['area_m2']/span, 'flaperon_'+side)
    for name, control in (('tail','elevator'),('fin','rudder')):
        f = s[name]
        span = math.sqrt(f['area_m2']*f['aspect'])
        airfoil(name, f, span, f['area_m2']/span, control)
    for i,g in enumerate(s['gear']):
        at = g['at']
        rings('gear_'+str(i), [(0,.055,.055),(.90,.055,.055)], 'metal',
              at=(at[0],at[1]+.20,at[2]), axis='Y', sides=6)
        rings('wheel_'+str(i), [(-.11,.23,.23),(.11,.23,.23)], 'ink',
              at=(at[0],at[1]+.23,at[2]), axis='X', sides=12)
    for side, sign in (('left',-1),('right',1)):
        at = s['rotor']['at']
        nac = node('nacelle_'+side, at=(sign*at[0],at[1],at[2]))
        rings('nacelle_skin_'+side, [(-1.25,.23,.23),(-.95,.37,.37),(.35,.35,.35),(.65,.23,.23)],
              'coral', nac, axis='Y', sides=10)
        rings('nacelle_collar_'+side, [(.32,.36,.36),(.47,.30,.30)], 'white', nac, axis='Y')
        rotor = node('rotor_'+side, nac, (0,.8,0))
        rings('rotor_hub_'+side, [(-.08,.18,.18),(.08,.18,.18),(.14,.06,.06)],
              'metal', rotor, axis='Y', sides=8)
        radius = s['rotor']['radius_m']
        tip = math.sqrt(radius*radius-.06*.06)
        for j in range(3):
            angle = j*2*math.pi/3
            outline = [(-.07,.18),(-.12,.65*radius),(-.06,tip),(.06,tip),(.10,.6*radius),(.06,.18)]
            outline = [(x*math.cos(angle)+z*math.sin(angle),-x*math.sin(angle)+z*math.cos(angle)) for x,z in outline]
            prism('blade_'+side+'_'+str(j), outline, .045, 'navy', rotor)
    crew(s['seat']['eye'])


def fixed_foil(name, spec, parent=None, at=None, thickness=.06):
    frame=node(name,parent,spec['at'] if at is None else at,foil_frame(spec))
    span=math.sqrt(spec['area_m2']*spec['aspect'])
    chord=spec['area_m2']/span
    prism(name+'_skin',[(-span/2,-chord/2),(span/2,-chord/2),
                       (span/2,chord/2),(-span/2,chord/2)],thickness,'navy',frame)
    return frame


def rod(name, start, end, radius, tag, parent=None):
    start,end=Vector(start),Vector(end)
    z=(end-start).normalized()
    x=z.cross(Vector((0,1,0)))
    if x.length<.001: x=Vector((1,0,0))
    x.normalize(); y=z.cross(x).normalized()
    basis=Matrix((x,y,z)).transposed()
    obj=rings(name,[(0,radius,radius),((end-start).length,radius,radius)],tag,parent,start,sides=6)
    obj.rotation_mode='QUATERNION'
    obj.rotation_quaternion=(C@basis@C.inverted()).to_quaternion()
    return obj


def boat_hull(kind, tag):
    h=DATA['hulls'][kind]
    faces=[h['indices'][i:i+3] for i in range(0,len(h['indices']),3)]
    poly('hull',h['positions'],faces,tag,closed=False)
    return h


def gunwales(h, tag):
    for side,index in (('left',0),('right',h['across'])):
        points=[]
        for station in range(h['stations']+1):
            x,y,z=h['positions'][station*(h['across']+1)+index]
            points += [(x,y-.025,z),(x,y+.035,z),(x*.94,y+.035,z),(x*.94,y-.025,z)]
        faces=[]
        for station in range(h['stations']):
            for j in range(4):
                a=station*4+j; b=station*4+(j+1)%4
                faces.append((a,b,b+4,a+4))
        faces.extend([(0,1,2,3),tuple(h['stations']*4+j for j in range(4))])
        poly('gunwale_'+side,points,faces,tag)


def build_tern():
    s=reset('tern')
    h=boat_hull('tern','white')
    gunwales(h,'pale')
    # Deck follows the authoritative rim, with a real recessed cockpit opening.
    deck=[]; faces=[]; hole=[]
    half=.52; seat_z=s['seat']['eye'][2]
    for i in range(h['stations']):
        l0,r0,l1,r1=h['deck'][2*i:2*i+4]
        cockpit=l0[2]>=seat_z-.85 and l1[2]<=seat_z+.85
        regions=[(l0,r0,l1,r1)]
        if cockpit:
            a=[-half,l0[1],l0[2]]; b=[half,r0[1],r0[2]]
            c=[-half,l1[1],l1[2]]; d=[half,r1[1],r1[2]]
            regions=[(l0,a,l1,c),(b,r0,d,r1)]
            hole.extend((l0[2],l1[2]))
        for a,b,c,d in regions:
            k=len(deck); deck.extend((a,b,c,d)); faces.append((k,k+2,k+3,k+1))
    poly('deck',deck,faces,'coral',closed=False)
    z0,z1=min(hole),max(hole); y=s['hull']['sheer_m']
    box('cockpit_floor',(half*2,.055,z1-z0),(0,y-.32,(z0+z1)/2),'navy')
    for side in (-1,1):
        box('cockpit_side_'+str(side),(.055,.32,z1-z0),(side*half,y-.16,(z0+z1)/2),'coral')
    for end,z in (('front',z0),('rear',z1)):
        box('cockpit_'+end,(half*2,.32,.055),(0,y-.16,z),'coral')
    box('cockpit_seat',(half*2,.09,.30),(0,y-.08,seat_z+.12),'pale')
    fixed_foil('keel',s['keel'])
    rings('ballast',[(-.54,.02,.02),(-.33,.17,.14),(.33,.17,.14),(.54,.02,.02)],
          'navy',at=s['parts'][1]['at'],sides=8)
    sail=s['sail']; mast=Vector(sail['mast']); head=sail['head_height_m']
    rings('mast',[(0,.06,.06),(head,.035,.035)],'metal',at=mast,axis='Y',sides=8)
    boom=node('boom',at=mast+Vector((0,sail['boom_height_m'],0)))
    length=sail['boom_length_m']; rise=head-sail['boom_height_m']
    rings('boom_skin',[(0,.045,.045),(length,.045,.045)],'metal',boom,sides=8)
    foot=2*sail['area_m2']/rise
    poly('sail',[(0,0,0),(0,rise,0),(0,0,foot)],[(0,1,2)],'pale',boom,closed=False)
    for side in (-1,1):
        rod('shroud_'+str(side),(side*s['hull']['beam_m']*.42,y,-.35),mast+Vector((0,head*.82,0)),.012,'navy')
    rudder=node('rudder',at=s['rudder']['at'])
    fixed_foil('rudder_foil',s['rudder'],rudder,(0,0,0),.055)
    tiller=node('tiller',at=s['rudder']['at'])
    handle_y=y-s['rudder']['at'][1]+.15
    rod('rudder_stock',(0,0,0),(0,handle_y,0),.028,'metal',rudder)
    rod('tiller_handle',(0,handle_y,0),(0,handle_y,-1.05),.035,'pale',tiller)
    crew(s['seat']['eye'])


def build_loon():
    s=reset('loon')
    h=boat_hull('loon','teal')
    # An inward-facing lining makes the canoe a genuinely open shell.
    inner=[]
    row=h['across']+1
    for i,(x,y,z) in enumerate(h['positions']):
        half=max(abs(h['positions'][(i//row)*row][0]),.001)
        lift=.025*max(0,1-(x/half)**2)
        inner.append((x*.94,y+lift,z))
    faces=[list(reversed(h['indices'][i:i+3])) for i in range(0,len(h['indices']),3)]
    poly('lining',inner,faces,'teal',closed=False)
    gunwales(h,'pale')
    sheer=s['hull']['sheer_m']
    for station in (5,10,15):
        half=abs(h['positions'][station*row][0])
        z=h['positions'][station*row][2]
        box('thwart_'+str(station),(half*1.87,.045,.09),(0,sheer-.06,z),'pale')
    seat_z=s['seat']['eye'][2]
    station=min(range(h['stations']+1),key=lambda i:abs(h['positions'][i*row][2]-seat_z))
    half=abs(h['positions'][station*row][0])
    box('seat',(half*1.6,.04,.31),(0,s['seat']['eye'][1]-.99,seat_z),'pale')
    # Small end caps leave the middle open and give the bow a readable badge.
    for end,station in (('bow',1),('stern',h['stations']-1)):
        x,y,z=h['positions'][station*row]
        tip=h['positions'][0 if end=='bow' else h['stations']*row]
        poly(end+'_cap',[(x,y+.012,z),(-x,y+.012,z),(0,sheer+.012,tip[2])],
             [(0,1,2) if end=='bow' else (0,2,1)],'teal',closed=False)
    fixed_foil('skeg',s['skeg'],thickness=.035)
    paddle=node('paddle')
    height=.45
    width=s['paddle']['blade_m2']/(.94*height)
    outline=[(-height/2,-.3*width),(-.35*height,-width/2),(.35*height,-width/2),
             (height/2,-.3*width),(height/2,.3*width),(.35*height,width/2),
             (-.35*height,width/2),(-height/2,.3*width)]
    n=len(outline)
    points=[(x,y,z) for x in (-.01,.01) for y,z in outline]
    faces=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]
    faces += [(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    poly('paddle_blade',points,faces,'pale',paddle)
    rings('paddle_shaft',[(0,.019,.019),(1.30,.019,.019)],'pale',paddle,axis='Y',sides=8)
    rings('paddle_grip',[(-.11,.025,.025),(.11,.025,.025)],'navy',paddle,at=(0,1.30,0),sides=8)
    crew(s['seat']['eye'])


def read_swatch(name):
    """Read normalized RGBA8 PNG without Pillow in Blender's Python."""
    if name in SWATCHES: return SWATCHES[name]
    raw=(ROOT/'swatches'/(name+'.png')).read_bytes()
    offset=8; compressed=b''; width=height=0
    while offset<len(raw):
        length=struct.unpack_from('>I',raw,offset)[0]; kind=raw[offset+4:offset+8]; data=raw[offset+8:offset+8+length]
        if kind==b'IHDR':
            width,height,depth,mode,_,_,interlace=struct.unpack('>IIBBBBB',data)
            assert (width,height,depth,mode,interlace)==(32,32,8,6,0)
        if kind==b'IDAT': compressed+=data
        offset+=length+12
    data=zlib.decompress(compressed); stride=width*4; previous=bytearray(stride); rows=[]
    for y in range(height):
        start=y*(stride+1); mode=data[start]; row=bytearray(data[start+1:start+1+stride])
        for x in range(stride):
            a=row[x-4] if x>=4 else 0; b=previous[x]; c=previous[x-4] if x>=4 else 0
            prediction=0
            if mode==1: prediction=a
            elif mode==2: prediction=b
            elif mode==3: prediction=(a+b)//2
            elif mode==4:
                p=a+b-c; distances=(abs(p-a),abs(p-b),abs(p-c));prediction=(a,b,c)[distances.index(min(distances))]
            elif mode!=0: raise ValueError('PNG filter')
            row[x]=(row[x]+prediction)&255
        rows.append(row);previous=row
    pixels=b''.join(reversed(rows));SWATCHES[name]=pixels
    return pixels


def sample_swatch(name, u, v):
    USED_SWATCHES.add(name)
    pixels=read_swatch(name); offset=((math.floor(v)%32)*32+math.floor(u)%32)*4
    return pixels[offset:offset+4]


def painted_tile(chart, obj, u_axis, v_axis, lo, plane):
    """Generated material imagery plus deliberately placed craft-specific marks."""
    name,tag=obj.name,chart['tag'];w,h=chart['size']
    wood=name.startswith(('tiller_handle','thwart_','gunwale_')) or name=='seat'
    material={'white':'white-panels','coral':'coral-trim','navy':'rotor','ink':'rotor',
              'metal':'gear-metal','pale':'white-panels','amber':'coral-trim','teal':'teal-hull'}[tag]
    if wood: material='varnished-wood'
    if name=='canopy': material='canopy'
    if KIND=='tern':
        if name=='hull': material='white-topsides'
        if name=='deck' or name.startswith('cockpit_'): material='coral-deck'
        if name=='sail': material='sail-cloth'
        if tag=='navy': material='antifouling'
    if KIND=='loon':
        if name=='lining': material='inner-ribs'
        if name.startswith('paddle_') and tag=='pale': material='pale-paddle'
    chart['swatch']=material
    result=bytearray();stencil=('101000111','110000010','100111010','110000010','101000111')
    for v in range(h):
        for u in range(w):
            local=u_axis*((u+.5+lo[0])/DENSITY)+v_axis*((v+.5+lo[1])/DENSITY)+plane
            p=C.inverted()@local; world=C.inverted()@(obj.matrix_world@local)
            su,sv=u,v
            if name=='deck': su,sv=world.x*16,world.z*16
            if name in ('hull','lining'): su,sv=world.z*16,world.y*16
            if name=='lining': su,sv=world.y*16,world.z*16
            if name=='sail': su,sv=p.z*16,p.y*16
            color=sample_swatch(material,su,sv)
            if tag=='amber':
                index=RAMPS['coral'].index(color[:3].hex().upper())
                color=bytes(rgba(RAMPS['amber'][index]))
            # Unique trim, safety and identification marks. Swatches supply all
            # ordinary material seams, rivets, grille slots, grain and cloth.
            if tag=='coral' and (u==w-2 or v==h-2): color=bytes(rgba(RAMPS['coral'][2]))
            if KIND=='kestrel':
                if name.startswith('wing_') and abs(p.x)>1.65 and abs(world.x)<1.0:
                    color=bytes(rgba(RAMPS['navy'][1 if (u+v)%4 else 2]))
                if name=='canopy':
                    if u<1 or v<1 or u>=w-1 or v>=h-1: color=bytes(rgba(RAMPS['navy'][0]))
                    if w>5 and h>4 and v==h-3 and u in (2,3): color=bytes(rgba(RAMPS['white'][3]))
                if name=='cabin_hatch' and w>=12 and h>=7:
                    sx,sy=u-3,v-2
                    if 0<=sx<9 and 0<=sy<5 and stencil[4-sy][sx]=='1': color=bytes(rgba(RAMPS['navy'][0]))
                if name.startswith('nacelle_skin'):
                    if -.8<p.y<-.25: color=sample_swatch('intake-grille',u,p.y*16)
                    if .15<p.y<.35: color=bytes(rgba(RAMPS['amber'][2] if (u+v)//2%2 else RAMPS['navy'][0]))
                if name.startswith('blade_') and math.hypot(p.x,p.z)>1.7: color=bytes(rgba(RAMPS['coral'][2]))
            if KIND=='tern' and name=='hull':
                if world.y<-.0625: color=sample_swatch('antifouling',su,sv)
                elif world.y<.0625: color=bytes(rgba(RAMPS['coral'][1]))
            if KIND=='loon':
                if name.startswith('thwart_') and (u<3 or u>=w-3): color=bytes(rgba(RAMPS['pale'][0 if u%2 else 2]))
                if name=='paddle_blade' and (abs(p.y)>.16 or abs(p.z)>.085): color=bytes(rgba(RAMPS['wood'][1]))
            result.extend(color)
    # Add a stitch/fastener pair only when a small isolated face samples a calm
    # swatch patch; do not replace the generated material's ordinary details.
    if w>=5 and h>=5 and len(set(tuple(result[i:i+4]) for i in range(0,len(result),4)))==1:
        cx=max(1,min(w-2,int(sum(p[0] for p in chart['coords'])/len(chart['coords']))))
        cy=max(1,min(h-2,int(sum(p[1] for p in chart['coords'])/len(chart['coords']))))
        ramp=RAMPS['wood' if wood else 'navy' if tag=='ink' else tag]
        mark=bytes(rgba(ramp[2] if result[:4]==bytes(rgba(ramp[0])) else ramp[0]))
        for x in (cx,cx+1): result[(cy*w+x)*4:(cy*w+x+1)*4]=mark
    return bytes(result)


def pack_tiles(tiles):
    """Deterministic best-short-side rectangle packing, with quarter turns."""
    ordered = sorted(range(len(tiles)), key=lambda i:(-max(tiles[i]['size']),-math.prod(tiles[i]['size']),i))
    def attempt(width,height):
        free = [(0,0,width,height)]; placements = {}
        for i in ordered:
            w,h = (v+2*PADDING for v in tiles[i]['size'])
            choices = []
            for x,y,fw,fh in free:
                for rw,rh,turn in ((w,h,False),(h,w,True)):
                    if rw <= fw and rh <= fh:
                        choices.append((min(fw-rw,fh-rh),max(fw-rw,fh-rh),y,x,rw,rh,turn))
            if not choices: return None
            _,_,y,x,rw,rh,turn = min(choices)
            placements[i] = (x,y,rw,rh,turn)
            remaining = []
            for a,b,c,d in free:
                if x >= a+c or x+rw <= a or y >= b+d or y+rh <= b:
                    remaining.append((a,b,c,d)); continue
                if x > a: remaining.append((a,b,x-a,d))
                if x+rw < a+c: remaining.append((x+rw,b,a+c-x-rw,d))
                if y > b: remaining.append((a,b,c,y-b))
                if y+rh < b+d: remaining.append((a,y+rh,c,b+d-y-rh))
            free = [r for j,r in enumerate(remaining) if not any(k != j and
                    r[0]>=s[0] and r[1]>=s[1] and r[0]+r[2]<=s[0]+s[2] and r[1]+r[3]<=s[1]+s[3]
                    and (r!=s or k<j) for k,s in enumerate(remaining))]
        return placements
    area = sum((t['size'][0]+2*PADDING)*(t['size'][1]+2*PADDING) for t in tiles)
    candidates = sorted((max(w,h)>256,w*h,max(w,h),w,h) for w in range(64,513,16) for h in range(w,513,16) if w*h>=area)
    for _,_,_,width,height in candidates:
        placements = attempt(width,height)
        if placements is not None: return width,height,placements
    raise RuntimeError('Atlas overflow; do not rescale UVs')


def png(path, pixels, width, height):
    def chunk(kind, data):
        return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
    raw = b''.join(b'\x00'+pixels[y*width*4:(y+1)*width*4] for y in reversed(range(height)))
    path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))+
                     chunk(b'IDAT',zlib.compress(raw,9))+chunk(b'IEND',b''))


def texture_and_export():
    folder = ROOT / KIND
    folder.mkdir(parents=True, exist_ok=True)
    bpy.context.view_layer.update()
    charts,tiles,shared = [],[],{}
    for obj in SCENE.objects:
        if obj.type != 'MESH': continue
        mesh = obj.data
        uv = mesh.uv_layers.new(name='Pixel16')
        for face in mesh.polygons:
            points = [mesh.vertices[i].co for i in face.vertices]
            edges = [points[(j+1)%len(points)]-p for j,p in enumerate(points)]
            u = max(edges,key=lambda p:p.length).normalized()
            if obj.name == 'sail': u = C @ Vector((0,0,1))
            if obj.name == 'deck': u = C @ Vector((1,0,0))
            if obj.name in ('hull','lining'):
                axis = C @ Vector((0,0,1))
                u = (axis-face.normal*axis.dot(face.normal)).normalized()
            v = face.normal.cross(u).normalized()
            coords = [(p.dot(u)*DENSITY,p.dot(v)*DENSITY) for p in points]
            lo = (math.floor(min(p[0] for p in coords)),math.floor(min(p[1] for p in coords)))
            coords = [(x-lo[0],y-lo[1]) for x,y in coords]
            w = math.ceil(max(p[0] for p in coords))+1
            h = math.ceil(max(p[1] for p in coords))+1
            chart = {'object':obj.name,'face':face.index,'tag':TAGS[obj.name],
                     'size':[w,h],'coords':coords,'loops':list(face.loop_indices),'uv':uv}
            pixels = painted_tile(chart,obj,u,v,lo,face.normal*points[0].dot(face.normal))
            key = (w,h,pixels)
            if key not in shared:
                shared[key] = len(tiles)
                tiles.append({'size':[w,h],'pixels':pixels,'users':[]})
            chart['tile'] = shared[key]
            tiles[chart['tile']]['users'].append({k:chart[k] for k in ('object','face','tag','swatch')})
            charts.append(chart)
    width,height,placements = pack_tiles(tiles)
    pixels = bytearray(rgba(RAMPS['navy'][0])*(width*height))
    regions = []
    for i,tile in enumerate(tiles):
        x,y,rw,rh,turn = placements[i]
        w,h = tile['size']; sw,sh = (h,w) if turn else (w,h)
        for py in range(rh):
            for px in range(rw):
                a,b = max(0,min(sw-1,px-PADDING)),max(0,min(sh-1,py-PADDING))
                u,v = (b,a) if turn else (a,b)
                start = (v*w+u)*4; dest = ((y+py)*width+x+px)*4
                pixels[dest:dest+4] = tile['pixels'][start:start+4]
        regions.append({'users':tile['users'],'size':[sw,sh], 'rect':[x,y,rw,rh],
                        'origin':[x+PADDING,y+PADDING],'padding':PADDING})
    for chart in charts:
        r = regions[chart['tile']]; ox,oy = r['origin']; turn = placements[chart['tile']][4]
        for loop,(u,v) in zip(chart['loops'],chart['coords']):
            if turn: u,v = v,u
            chart['uv'].data[loop].uv = ((ox+u)/width,(oy+v)/height)
    png(folder/'atlas.png',pixels,width,height)
    palette = sorted(set(bytes(pixels[i:i+3]).hex().upper() for i in range(0,len(pixels),4)))
    assert 12 <= len(palette) <= 24, (KIND,len(palette))
    # Review image uses exact integer replication, never an interpolating resize.
    enlarged = bytearray()
    for y in range(height):
        row = b''.join(pixels[(y*width+x)*4:(y*width+x+1)*4]*3 for x in range(width))
        enlarged.extend(row*3)
    png(ROOT.parents[2]/'output'/'vehicles-improve'/(KIND+'-atlas-nearest.png'),enlarged,width*3,height*3)
    image=bpy.data.images.load(str(folder/'atlas.png'),check_existing=False)
    image.pack()
    mat=bpy.data.materials.new(KIND+'_pixel16')
    mat.use_nodes=True
    mat.use_backface_culling=True
    shader=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    shader.inputs['Roughness'].default_value=.72
    shader.inputs['Metallic'].default_value=0
    tex=mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex.image=image
    tex.interpolation='Closest'
    mat.node_tree.links.new(tex.outputs['Color'],shader.inputs['Base Color'])
    for obj in SCENE.objects:
        if obj.type=='MESH':
            surface=mat
            if obj.name=='sail':
                surface=mat.copy()
                surface.name=KIND+'_sail_pixel16'
                surface.use_backface_culling=False
            obj.data.materials.append(surface)
    manifest={'schema':1,'craft':KIND,'pixels_per_metre':DENSITY,'width':width,'height':height,
              'pixel_origin':'bottom-left, Blender UV; glTF V is flipped',
              'palette':{str(i):h for i,h in enumerate(palette)},'swatches':sorted(USED_SWATCHES),'regions':regions}
    (folder/'atlas.json').write_text(json.dumps(manifest,indent=2)+'\n')
    bpy.ops.export_scene.gltf(filepath=str(folder/(KIND+'.glb')),export_format='GLB',
        use_active_scene=True,export_yup=True,export_apply=False,export_animations=False,
        export_texcoords=True,export_normals=True,export_tangents=False,export_image_format='AUTO')
    bpy.data.libraries.write(str(folder/(KIND+'.blend')),{SCENE},path_remap='RELATIVE_ALL',compress=True)
    for area in bpy.context.screen.areas:
        if area.type=='VIEW_3D':
            space=area.spaces.active
            space.shading.type='MATERIAL'
            space.overlay.show_overlays=False
            space.region_3d.view_rotation=Euler((math.radians(65),0,math.radians(135))).to_quaternion()
            space.region_3d.view_location=C@Vector((0,3.1 if KIND=='tern' else .6,0))
            space.region_3d.view_distance=17 if KIND=='kestrel' else 12.5 if KIND=='tern' else 8
    print(json.dumps({'craft':KIND,'objects':len(SCENE.objects),'charts':len(charts),'tiles':len(regions),'colors':len(palette),'atlas':[width,height],
        'triangles':sum(len(p.vertices)-2 for o in SCENE.objects if o.type=='MESH' for p in o.data.polygons),
        'atlas_occupied_fraction':sum(r['rect'][2]*r['rect'][3] for r in regions)/(width*height)}))


def build(kind):
    if kind=='kestrel': build_kestrel()
    elif kind=='tern': build_tern()
    elif kind=='loon': build_loon()
    else: raise ValueError('Craft authoring is not implemented: '+kind)
    texture_and_export()
