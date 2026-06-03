import { HODTexture } from "./components/Viewport";

export interface TextureGroupItem {
  type: string; // "_DIFF", "_GLOW", "_TEAM", "_NORM", "_SPEC", etc. or ""
  originalName: string;
  texture: HODTexture;
  compression: string;
}

export interface TextureGroup {
  baseName: string;
  textures: TextureGroupItem[];
}

export const KNOWN_TYPES = ["_DIFF", "_GLOW", "_TEAM", "_NORM", "_SPEC", "_STRP", "_PAIN", "_MASK", "_EMIS"];

export function parseTextureGroups(textures: HODTexture[]): TextureGroup[] {
  const groups = new Map<string, TextureGroup>();

  for (const tex of textures) {
    let name = tex.name;
    // Strip compression if present at the end
    if (name.toUpperCase().endsWith(tex.format.toUpperCase())) {
      name = name.substring(0, name.length - tex.format.length);
    }

    let type = "";
    let baseName = name;

    // Find the known type
    for (const t of KNOWN_TYPES) {
      if (name.toUpperCase().endsWith(t)) {
        type = t;
        baseName = name.substring(0, name.length - t.length);
        break;
      }
    }

    // If no known type, maybe the whole name is the base name and type is empty
    if (!groups.has(baseName)) {
      groups.set(baseName, { baseName, textures: [] });
    }

    groups.get(baseName)!.textures.push({
      type,
      originalName: tex.name,
      texture: tex,
      compression: tex.format,
    });
  }

  return Array.from(groups.values());
}
