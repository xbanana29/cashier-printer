import base64, json, urllib.request, urllib.error, subprocess, sys

# Read license text
with open('/tmp/lgpl_body.txt') as f:
    text = f.read()

encoded = base64.b64encode(text.encode('utf-8')).decode('ascii')
payload = json.dumps({"message": "chore: add LGPL-2.1 license", "content": encoded}).encode('utf-8')

# Get token
token_res = subprocess.run(
    ["/c/Program Files/GitHub CLI/gh.exe", "auth", "token"],
    capture_output=True, text=True
)
token = token_res.stdout.strip()

req = urllib.request.Request(
    "https://api.github.com/repos/nikokevin29/VST-laravel/contents/LICENSE",
    data=payload,
    method="PUT",
    headers={
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28"
    }
)
try:
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read())
        print("OK:", data["commit"]["sha"][:12])
except urllib.error.HTTPError as e:
    print("Error:", e.code, e.read().decode())
    sys.exit(1)
