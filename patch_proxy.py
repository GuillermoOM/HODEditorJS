import re

with open("src/App.tsx", "r") as f:
    data = f.read()

old_proxy = """          const createProxyJoint = (childName: string, navName: string, prefix: string) => {
            const info = getNavlightInfo(navName);
            const newJointName = `${prefix}_${childName}_from_${navName}`;
            if (!m.joints.some(j => j.name === newJointName)) {
              m.joints.push({
                name: newJointName,
                parent_name: info.parentName,
                local_transform: JSON.parse(JSON.stringify(info.transform))
              });
              invoke("log_event", { level: "INFO", message: `Sanitized: Created proxy joint ${newJointName} to decouple ${childName} from navlight ${navName}` }).catch(console.error);
            }
            return newJointName;
          };"""

new_proxy = """          const createProxyJoint = (childName: string, navName: string, prefix: string) => {
            const info = getNavlightInfo(navName);
            let newJointName = `${prefix}_${childName}_from_${navName}`;
            
            if (prefix.includes("BurnProxy") || prefix.includes("GlowProxy") || prefix.includes("ShapeProxy")) {
                let idx = 0;
                while (m.joints.some(j => j.name === `EngineNozzle${idx}`)) idx++;
                newJointName = `EngineNozzle${idx}`;
            }

            if (!m.joints.some(j => j.name === newJointName)) {
              m.joints.push({
                name: newJointName,
                parent_name: info.parentName,
                local_transform: JSON.parse(JSON.stringify(info.transform))
              });
              invoke("log_event", { level: "INFO", message: `Sanitized: Created proxy joint ${newJointName} to decouple ${childName} from navlight ${navName}` }).catch(console.error);
            }
            return newJointName;
          };"""

if old_proxy in data:
    data = data.replace(old_proxy, new_proxy)
    print("Patched createProxyJoint!")
else:
    print("Could not find old createProxyJoint logic.")

with open("src/App.tsx", "w") as f:
    f.write(data)
