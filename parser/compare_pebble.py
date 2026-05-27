import struct
import sys

def parse_chunks(filepath):
    with open(filepath, 'rb') as f:
        data = f.read()
    
    pos = 0
    chunks = []
    while pos < len(data):
        if pos + 8 > len(data): break
        cid = data[pos:pos+4].decode('ascii', errors='ignore')
        size = struct.unpack('>I', data[pos+4:pos+8])[0]
        chunks.append((cid, size, data[pos+8:pos+8+size]))
        pos += 8 + size
    return chunks

def extract_mult(data):
    pos = 0
    mults = []
    while pos < len(data):
        if pos + 8 > len(data): break
        cid = data[pos:pos+4].decode('ascii', errors='ignore')
        size = struct.unpack('>I', data[pos+4:pos+8])[0]
        if cid == 'MULT':
            mults.append(data[pos+8:pos+8+size])
        elif cid == 'HVMD':
            mults.extend(extract_mult(data[pos+8:pos+8+size]))
        pos += 8 + size
    return mults

mults_orig = extract_mult(open('/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod', 'rb').read())
mults_edit = extract_mult(open('/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0_edited.hod', 'rb').read())

print("Original MULT chunks:", len(mults_orig))
for i, m in enumerate(mults_orig):
    print(f"  {i}: size={len(m)}, data={m[:32].hex()}")

print("Edited MULT chunks:", len(mults_edit))
for i, m in enumerate(mults_edit):
    print(f"  {i}: size={len(m)}, data={m[:32].hex()}")
