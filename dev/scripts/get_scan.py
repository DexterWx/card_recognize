import requests
import json
import sys
import os

# 改变当前工作目录为脚本所在目录
os.chdir(os.path.dirname(os.path.abspath(__file__)))

# 获取命令行参数
exam_id = sys.argv[1]
image_url = sys.argv[2]

# 构建URL和目录路径
url = f'https://scanstat.17zuoye.net/unipus_staging/exam/exam/rec_info?uid={exam_id}'
base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # 上一级目录
wdir = os.path.join(base_dir, 'test_data', 'cards', exam_id)
wpath = os.path.join(wdir, 'scan.json')
image_dir = os.path.join(wdir, 'images')
image_path = os.path.join(image_dir, 'test.jpg')

# 确保图片目录存在
if not os.path.exists(image_dir):
    os.makedirs(image_dir)

# 下载JSON文件并保存
data = requests.get(url).json()
with open(wpath, 'w') as f:
    json.dump(data['body']['data'], f)

# 下载图片并保存
image = requests.get(image_url)
with open(image_path, "wb") as f:
    f.write(image.content)