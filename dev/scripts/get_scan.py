import requests
import json
import sys
import os

url = sys.argv[1]
wdir = sys.argv[2]
wpath = wdir + '/scan.json'

os.system(f'mkdir -p {wdir+"/images"}')

# 下载json
data = requests.get(url).json()
with open(wpath, 'w') as f:
    json.dump(data['body']['data'],f)
