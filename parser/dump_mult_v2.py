import struct
import sys

def extract_mult(data):
    pos = 0
    while pos < len(data):
        if pos + 8 > len(data): break
        cid = data[pos:pos+4].decode('ascii', errors='ignore')
        size = struct.unpack('>I', data[pos+4:pos+8])[0]
        
        if cid == 'MULT':
            payload = data[pos+8:pos+8+size]
            print(f"Found MULT size {size}")
            p = 0
            if p + 4 <= len(payload):
                nl = struct.unpack('<I', payload[p:p+4])[0]
                p += 4
                name = payload[p:p+nl].decode('ascii', errors='ignore')
                p += nl
                print("  Name:", name)
                if p + 4 <= len(payload):
                    pl = struct.unpack('<I', payload[p:p+4])[0]
                    p += 4
                    parent = payload[p:p+pl].decode('ascii', errors='ignore')
                    p += pl
                    print("  Parent:", parent)
                    if p + 4 <= len(payload):
                        lod = struct.unpack('<I', payload[p:p+4])[0]
                        print("  LODs:", lod)
        
        if cid == 'HVMD' or cid == 'DTRM':
            extract_mult(data[pos+8:pos+8+size])
            
        pos += 8 + size

data = open(sys.argv[1], 'rb').read()
extract_mult(data)
