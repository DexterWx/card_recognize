import requests
import json
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

data = {
    "uid": 199611,
    "recResult": {
        "index1": [
            {
                "index1":1
            }
        ]
    },
    "recInitParam": "",
    "fillRate": 0.5,
}

with open("../test_data/199611.json") as f:
    data['recResult'] = json.load(f)

with open("../test_data/cards/199611/scan.json") as f:
    data['recInitParam'] = json.load(f)

url = "https://scanstat.17zuoye.net/unipus_staging/exam/exam/generate_scan_datas"

res = requests.post(url, json = data).text
res = json.loads(res)
with open("../test_data/199611_res.json", 'w') as f:
    json.dump(res, f)