
### Normal.W Handedness
The 4th component of the Normal vector (`Normal.W`) in `0x600B` vertices MUST be set to `1.0` (or the appropriate Tangent Handedness sign). If set to `0.0`, the vertex shader calculates a singular TBN matrix leading to NaNs and a massive vertex explosion in-game.

### Xpress Compression Limits
The Homeworld Remastered internal Xpress decompressor appears to fail or corrupt memory if fed a fully uncompressed stream disguised as an Xpress chunk, or if blocks exceed standard MS Xpress 64KB chunk boundaries without proper framing. Proper match-copy emission is required.
