import re

filepath = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/parser/src/hod.rs"

with open(filepath, 'r') as f:
    content = f.read()

# 1. Change 1024 to 8192
content = content.replace('png_data = encode_b64_png_thumbnail(&rgba, width, height, 1024);', 'png_data = encode_b64_png_thumbnail(&rgba, width, height, 8192);')

# 2. Change Nearest to Lanczos3
content = content.replace('image::imageops::FilterType::Nearest', 'image::imageops::FilterType::Lanczos3')

with open(filepath, 'w') as f:
    f.write(content)
