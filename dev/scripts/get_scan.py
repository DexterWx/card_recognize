import requests
import json
import sys
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

exam_id = sys.argv[1]

url = f'https://scanstat.17zuoye.net/unipus_staging/exam/exam/rec_info?uid={exam_id}'
wdir = f'../test_data/cards/{exam_id}'
wpath = wdir + '/scan.json'
image_dir = wdir+"/images"
    
if not os.path.exists(image_dir):
    os.makedirs(image_dir)

# 下载json
data = requests.get(url).json()
with open(wpath, 'w') as f:
    json.dump(data['body']['data'],f)


if sys.platform.startswith("win"):
    image_dir = image_dir.replace('/','\\')
    
print(image_dir)