> **ARCHIVED.** Historical record; not active guidance. See docs/README.md.

# DAEnerys Analysis: OBJ to DAE Ingress Pipeline

## Overview

DAEnerys is a C# Windows Forms application (built with OpenTK and Assimp) that serves as a DAE file editor for Homeworld Remastered modding. It creates DAE files that are then converted to HOD 2.0 format by RODOH/HODOR.

## How DAEnerys Ingresses .OBJ Files

### 1. OBJ Import Process (`ObjImporter.cs`)

DAEnerys uses the **Assimp library** (Open Asset Import Library) to import OBJ files:

```csharp
// Key import steps:
AssimpContext importer = new AssimpContext();
NormalSmoothingAngleConfig config = new NormalSmoothingAngleConfig(80.0f);
importer.SetConfig(config);

Scene obj = importer.ImportFile(path, 
    PostProcessSteps.JoinIdenticalVertices | 
    PostProcessSteps.Triangulate | 
    PostProcessSteps.ValidateDataStructure);
```

**Import Configuration:**
- **Normal smoothing angle**: 80.0 degrees
- **Post-processing steps**:
  - `JoinIdenticalVertices`: Removes duplicate vertices
  - `Triangulate`: Converts all polygons to triangles
  - `ValidateDataStructure`: Validates the imported mesh

### 2. Mesh Combination Logic

DAEnerys combines meshes with the same material into a single mesh:

```csharp
Dictionary<Material, Mesh> materialMeshes = new Dictionary<Material, Mesh>();
foreach (Mesh mesh in obj.Meshes)
{
    Material material = obj.Materials[mesh.MaterialIndex];
    
    if (!materialMeshes.ContainsKey(material))
    {
        materialMeshes.Add(material, new Mesh(PrimitiveType.Triangle));
    }
    
    // Combine vertices, normals, UVs, and indices
    materialMesh.Vertices.AddRange(mesh.Vertices);
    materialMesh.Normals.AddRange(mesh.Normals);
    materialMesh.TextureCoordinateChannels[0].AddRange(mesh.TextureCoordinateChannels[0]);
}
```

### 3. Vertex Data Structure (`GenericMesh.cs`)

Each vertex contains:
- **Position** (Vector3): X, Y, Z coordinates
- **Normal** (Vector3): Normal vector
- **Color** (Vector3): Vertex color
- **UV0** (Vector2): Primary texture coordinates
- **UV1** (Vector2): Secondary texture coordinates
- **Tangent** (Vector3): Tangent vector for normal mapping
- **BiTangent** (Vector3): Bitangent vector for normal mapping

### 4. Tangent/Bitangent Calculation

DAEnerys calculates tangents and bitangents for normal mapping:

```csharp
private void RecalculateTangents()
{
    foreach (Vertex vtx in VertexList)
    {
        vtx.Tangent = new Vector3();
        vtx.BiTangent = new Vector3();
    }
    
    // Calculate face tangents
    for (int i = 0; i < IndexCount; i += 3)
    {
        _CalcFaceTangents(out fT, out fB, out fH, i1, i2, i3);
        _UpdateVertTangents(altv, ref handedness, fT, fB, fH, i1, out N);
    }
    
    // Gram-Schmidt orthogonalization
    for (int i = 0; i < Vertices.Length; ++i)
        _NormaliseVertTangents(handedness[i], i);
}
```

## How DAEnerys Outputs .DAE Files

### 1. DAE File Structure (`Exporter.cs`)

DAEnerys generates COLLADA 1.4.1 format DAE files with the following structure:

```xml
<COLLADA version="1.4.1">
    <asset>
        <contributor>
            <authoring_tool>DAEnerys bXXX</authoring_tool>
        </contributor>
        <unit meter="1.0" unit="centimeter" />
        <up_axis>Y_UP</up_axis>
    </asset>
    
    <library_images>...</library_images>
    <library_materials>...</library_materials>
    <library_effects>...</library_effects>
    <library_geometries>...</library_geometries>
    <library_visual_scenes>...</library_visual_scenes>
</COLLADA>
```

### 2. Image Naming Convention

Images follow the Homeworld naming convention:
```
IMG[TextureName]_FMT[Format]
```

Example:
```xml
<image id="IMG[Pharos_DIFF]-image" name="IMG[Pharos_DIFF]_FMT[DXT1]">
    <init_from>Pharos_DIFF.TGA</init_from>
</image>
```

**Supported Formats:**
- `DXT1`: Compressed (no alpha)
- `DXT3`: Compressed (explicit alpha)
- `DXT5`: Compressed (interpolated alpha)
- `8888`: Uncompressed (RGBA)

### 3. Material Naming Convention

Materials follow the Homeworld shader convention:
```
MAT[TextureName]_SHD[ShaderType]
```

Example:
```xml
<material id="MAT[pharos.bmp]_SHD[ship]" name="MAT[pharos.bmp]_SHD[ship]">
    <instance_effect url="#MAT[pharos.bmp]_SHD[ship]-fx" />
</material>
```

### 4. Geometry Naming Convention

Meshes follow the Homeworld LOD convention:
```
MULT[MeshName]_LOD[Level]
```

Example:
```xml
<geometry id="MULT[Root_mesh]_LOD[0]-lib" name="MULT[Root_mesh]_LOD[0]Mesh">
    <mesh>...</mesh>
</geometry>
```

### 5. Visual Scene Hierarchy

The visual scene follows a specific hierarchy:

```xml
<visual_scene id="ter_pharos" name="ter_pharos">
    <node name="ROOT_COL" id="ROOT_COL">
        <node name="COL[Root]" id="COL[Root]">
            <instance_geometry url="#COL[Root]-lib" />
        </node>
    </node>
    
    <node name="ROOT_INFO" id="ROOT_INFO">
        <node name="UVSets[1]" id="UVSets[1]" />
        <node name="Class[MultiMesh]_Version[512]" id="Class[MultiMesh]_Version[512]" />
    </node>
    
    <node name="ROOT_LOD[0]" id="ROOT_LOD[0]">
        <node name="NAVL[Navlight_B1]" id="NAVL[Navlight_B1]">
            <node name="SUB_PARAMS" id="SUB_PARAMS">
                <node name="Sz[1]" id="Sz[1]" />
                <node name="Col[0,0,1]" id="Col[0,0,1]" />
            </node>
        </node>
        <node name="MULT[Root_mesh]_LOD[0]" id="MULT[Root_mesh]_LOD[0]">
            <instance_geometry url="#MULT[Root_mesh]_LOD[0]-lib">
                <bind_material>
                    <instance_material symbol="MAT[pharos.bmp]_SHD[ship]" />
                </bind_material>
            </instance_geometry>
        </node>
    </node>
</visual_scene>
```

## Texture Processing by RODOH/HODOR

### 1. Texture File Requirements

DAEnerys expects these texture files alongside the DAE:
- `Pharos_DIFF.TGA`: Diffuse texture
- `Pharos_TEAM.TGA`: Team color mask
- `Pharos_GLOW.TGA`: Glow/emission texture
- `Pharos_SPEC.TGA`: Specular texture
- `Pharos_STRP.TGA`: Stripe mask

### 2. Shader Mapping (`SHADERS.MAP`)

RODOH/HODOR uses the `SHADERS.MAP` file to map texture channels to game shaders:

```
+ship,matte,matte2s
    $diffuse[DXT1] = 1 1 1 1
        DIFF = R G B 1
    $glow[DXT1]= 0 0 0 1
        GLOW = G G G G
        SPEC = B B B B
        REFL = R R R R
    $team[DXT1] = 1 1 0 1
        TEAM = 1 1 1 r
        STRP = 1 1 1 g
        PAIN = 1 1 1 b
    $normal[DXT1]= 5 5 1 1
        NORM[B] = R G B 1
```

### 3. Texture Compression

RODOH/HODOR compresses TGA textures to DXT format:
- **DXT1**: For diffuse, glow, specular (no alpha)
- **DXT5**: For textures with alpha channel

## Workflow Summary

1. **OBJ Import**: DAEnerys imports OBJ files using Assimp
2. **Mesh Processing**: Combines meshes by material, calculates tangents
3. **DAE Export**: Generates COLLADA 1.4.1 DAE file with Homeworld naming conventions
4. **Texture Setup**: TGA textures are placed alongside DAE file
5. **HOD Conversion**: RODOH/HODOR converts DAE + TGA to HOD 2.0

## Key Observations

1. **Material-First Mesh Combination**: Meshes are combined based on material, not by name
2. **Tangent Calculation**: DAEnerys calculates tangents for normal mapping
3. **Naming Conventions**: All names follow strict Homeworld conventions
4. **LOD System**: Supports up to 3 LOD levels per mesh
5. **Shader Integration**: Materials are linked to Homeworld shaders via naming

## Technical Details

- **Language**: C# (.NET Framework)
- **3D Framework**: OpenTK (OpenGL wrapper)
- **Import Library**: Assimp (Open Asset Import Library)
- **File Format**: COLLADA 1.4.1
- **Platform**: Windows Forms
