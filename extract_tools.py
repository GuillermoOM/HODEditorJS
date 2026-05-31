import json

transcript_file = "/home/guillermo/.gemini/antigravity-cli/brain/0ad46c82-6024-4d0a-92b3-badba115868c/.system_generated/logs/transcript.jsonl"

with open(transcript_file, "r") as f:
    for line in f:
        data = json.loads(line)
        if "tool_calls" in data and data["tool_calls"]:
            for call in data["tool_calls"]:
                name = call.get("name")
                args = call.get("arguments", "{}")
                print(f"Tool: {name}")
                if name in ["multi_replace_file_content", "replace_file_content", "write_to_file", "run_command"]:
                    print(str(args)[:500])
