import os
import requests

def test_zone(zone):
    env_path = '.env'
    if os.path.exists(env_path):
        with open(env_path, 'r') as f:
            for line in f:
                if '=' in line:
                    k, v = line.strip().split('=', 1)
                    os.environ[k] = v.strip('"\'')

    api_key = os.environ.get("ELECTRICITY_MAPS_KEY")
    if not api_key:
        print("No key")
        return
        
    url = f"https://api.electricitymap.org/v3/carbon-intensity/latest?zone={zone}"
    resp = requests.get(url, headers={"auth-token": api_key})
    print(f"Zone {zone}: Status {resp.status_code}")
    if resp.status_code == 200:
        print(resp.json())
    else:
        print(resp.text)

test_zone("BR")
test_zone("BR-CS")
test_zone("BR-N")
test_zone("BR-PA")
