import fs from "fs";
const data = fs.readFileSync("/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_fenris/ter_fenris.hod");
// We can't parse HOD easily in JS, let's use the Rust parser via a rust script.
