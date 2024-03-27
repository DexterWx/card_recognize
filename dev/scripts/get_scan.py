import requests
import json
import sys
import os

url = sys.argv[1]
wdir = sys.argv[2]

wpath = wdir + '/scan.json'
image_dir = wdir+"/images"
if sys.platform.startswith("win"):
    wpath = wpath.replace('\\','/')
    image_dir = image_dir.replace('\\','/')
if not os.path.exists(image_dir):
    os.makedirs(image_dir)

# 下载json
data = requests.get(url).json()
with open(wpath, 'w') as f:
    json.dump(data['body']['data'],f)
