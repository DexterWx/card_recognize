import requests
import json
import sys
import os
import base64
from PIL import Image
from io import BytesIO

os.chdir(os.path.dirname(os.path.abspath(__file__)))

def image_to_base64(image_path):
    # 打开图片
    with Image.open(image_path) as image:
        # 创建一个字节流缓冲区
        buffered = BytesIO()
        # 将图片保存到字节流缓冲区中，格式为PNG（或其他格式）
        image.save(buffered, format="JPEG")
        # 获取字节流的字节数据
        img_bytes = buffered.getvalue()
        # 将字节数据编码为Base64字符串
        img_base64 = base64.b64encode(img_bytes).decode("utf-8")
        img_base64 = 'data:image/jpeg;base64,' + img_base64
        return img_base64

exam_id = sys.argv[1]

wdir = f'../test_data/cards/{exam_id}'
wpath = wdir + '/scan.json'
image_dir = wdir+"/images"
outpath = wdir + "/scan_second.json"

pages = []
imgs = []

for img_path in os.listdir(image_dir):
    imgs.append(image_to_base64(os.path.join(image_dir,img_path)))

with open(wpath) as f:
    scan_data = json.load(f)
    pages = scan_data['pages']

input = {
    "task_id":"",
    "pages":pages,
    "images": imgs
}

with open(outpath, 'w') as f:
    json.dump(input, f)




