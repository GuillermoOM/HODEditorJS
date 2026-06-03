import re

filepath = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/parser/src/hod.rs"

with open(filepath, 'r') as f:
    content = f.read()

funcs_to_remove = [
    "fn rgb565_to_u16",
    "fn u16_to_rgb565",
    "fn color_error",
    "fn find_best_endpoints",
    "fn compress_dxt1_block",
    "fn compress_dxt5_block",
    "fn compress_dxt3_block"
]

for func in funcs_to_remove:
    start_idx = content.find(func)
    if start_idx != -1:
        # Find the matching closing brace
        brace_count = 0
        in_func = False
        end_idx = start_idx
        for i in range(start_idx, len(content)):
            if content[i] == '{':
                brace_count += 1
                in_func = True
            elif content[i] == '}':
                brace_count -= 1
            
            if in_func and brace_count == 0:
                end_idx = i + 1
                break
        
        content = content[:start_idx] + content[end_idx:]

with open(filepath, 'w') as f:
    f.write(content)
