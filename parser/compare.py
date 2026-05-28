import struct
import zlib # Not needed

def read_pool_mesh(filepath):
    with open(filepath, "rb") as f:
        data = f.read()
    
    pos = 12
    while pos + 8 <= len(data):
        chunk_id = data[pos:pos+4].decode('utf-8', 'ignore')
        chunk_size = struct.unpack("<I", data[pos+4:pos+8])[0]
        if chunk_id == "POOL":
            pool_data = data[pos+8:pos+8+chunk_size]
            comp_tex = struct.unpack("<I", pool_data[4:8])[0]
            mesh_offset = 12 + comp_tex
            comp_mesh = struct.unpack("<I", pool_data[mesh_offset:mesh_offset+4])[0]
            decomp_mesh = struct.unpack("<I", pool_data[mesh_offset+4:mesh_offset+8])[0]
            print(f"{filepath} mesh_pool: comp={comp_mesh}, decomp={decomp_mesh}")
            
            comp_mesh_data = pool_data[mesh_offset+8:mesh_offset+8+comp_mesh]
            with open(filepath + ".mesh.comp", "wb") as out:
                out.write(comp_mesh_data)
            return
        pos += 8 + chunk_size

read_pool_mesh("/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod")
read_pool_mesh("/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod")
