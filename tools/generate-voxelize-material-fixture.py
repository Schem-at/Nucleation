#!/usr/bin/env python3
"""Generate our tiny material/skin regression GLB; no third-party model data."""
import json, pathlib, struct, zlib
blob = bytearray()
doc = {"asset":{"version":"2.0"},"extensionsUsed":["KHR_materials_transmission","KHR_materials_emissive_strength"],"bufferViews":[],"accessors":[]}
def view(data):
    while len(blob)%4: blob.append(0)
    i=len(doc['bufferViews']); doc['bufferViews'].append({'buffer':0,'byteOffset':len(blob),'byteLength':len(data)}); blob.extend(data); return i
def accessor(values,fmt,typ,component,bounds=False):
    n=len(values[0]); flat=[x for row in values for x in row]; i=len(doc['accessors']); a={'bufferView':view(struct.pack('<'+fmt*len(flat),*flat)),'componentType':component,'count':len(values),'type':typ}
    if bounds: a.update(min=[min(row[c] for row in values) for c in range(n)],max=[max(row[c] for row in values) for c in range(n)])
    doc['accessors'].append(a); return i
def png(rgba):
    def chunk(t,b): return struct.pack('>I',len(b))+t+b+struct.pack('>I',zlib.crc32(t+b))
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',1,1,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\0'+bytes(rgba)))+chunk(b'IEND',b'')
doc['images']=[{'bufferView':view(png(c)),'mimeType':'image/png'} for c in [(255,255,255,0),(0,128,255,255),(255,0,0,255),(128,128,128,255)]]
doc['textures']=[{'source':i} for i in range(4)]
doc['materials']=[
 {'pbrMetallicRoughness':{'baseColorTexture':{'index':0}}},
 {'alphaMode':'MASK','alphaCutoff':0.5,'pbrMetallicRoughness':{'baseColorTexture':{'index':0}}},
 {'alphaMode':'BLEND','pbrMetallicRoughness':{'baseColorFactor':[1,1,1,0]}},
 {'alphaMode':'BLEND','pbrMetallicRoughness':{'baseColorFactor':[1,1,1,0.25]}},
 {'extensions':{'KHR_materials_transmission':{'transmissionFactor':1,'transmissionTexture':{'index':2,'texCoord':1}}}},
 {'emissiveFactor':[1,1,1],'emissiveTexture':{'index':1,'texCoord':1},'extensions':{'KHR_materials_emissive_strength':{'emissiveStrength':2}},'pbrMetallicRoughness':{'baseColorFactor':[0,0,0,1]}},
 {'pbrMetallicRoughness':{'baseColorFactor':[0.5,0.5,0.5,1],'baseColorTexture':{'index':3}}},
 {'pbrMetallicRoughness':{'baseColorFactor':[1,0,0,1]}}
]
prims=[]
for x,mat,z in [(i*6,i,0) for i in range(7)]+[(42,1,0.25),(42,7,0)]:
    p=[(x,0,z),(x+4,0,z),(x+4,4,z),(x,4,z)];
    attrs={'POSITION':accessor(p,'f','VEC3',5126,True),'TEXCOORD_0':accessor([(0.5,0.5)]*4,'f','VEC2',5126),'TEXCOORD_1':accessor([(0.5,0.5)]*4,'f','VEC2',5126),'JOINTS_0':accessor([(0,0,0,0)]*4,'H','VEC4',5123),'WEIGHTS_0':accessor([(1,0,0,0)]*4,'f','VEC4',5126)}
    prims.append({'attributes':attrs,'indices':accessor([(i,) for i in [0,1,2,0,2,3]],'H','SCALAR',5123),'material':mat})
ibm=accessor([(1,0,0,0,0,1,0,0,0,0,1,0,-1,-2,-3,1)],'f','MAT4',5126)
doc.update(meshes=[{'primitives':prims}],nodes=[{'mesh':0,'skin':0,'translation':[100,0,0]},{'translation':[2,3,4]},{'mesh':0,'translation':[1000,0,0]}],skins=[{'joints':[1],'inverseBindMatrices':ibm}],scenes=[{'nodes':[0,1]},{'nodes':[2]}],scene=0,buffers=[{'byteLength':len(blob)}])
j=json.dumps(doc,separators=(',',':')).encode();j+=b' '*((-len(j))%4);blob+=b'\0'*((-len(blob))%4)
out=struct.pack('<III',0x46546c67,2,28+len(j)+len(blob))+struct.pack('<II',len(j),0x4e4f534a)+j+struct.pack('<II',len(blob),0x004e4942)+blob
pathlib.Path('tests/fixtures/voxelize-materials.glb').write_bytes(out)
