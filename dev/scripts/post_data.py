import requests
import json
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

data = {
    "uid": 197864,
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

with open("../test_data/197864.json") as f:
    data['recResult'] = json.load(f)

with open("../test_data/cards/197864/scan.json") as f:
    data['recInitParam'] = json.load(f)

with open("../test_data/197864_post.json",'w') as f:
    json.dump(data,f)

url = "https://scanstat.17zuoye.net/unipus_staging/exam/exam/generate_scan_datas"

res = requests.post(url, json = data).text
res = json.loads(res)
with open("../test_data/197864_res.json", 'w') as f:
    json.dump(res, f)