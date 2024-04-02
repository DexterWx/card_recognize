const { initialize, inference, test_image, ImageData, ImageInput } = require('..\\..\\pkg\\card_recognize.js');
const path = require('path');
const fs = require('fs');

// 读取图像文件并转换为字节数组
const filePath1 = path.join(__dirname, '../test_data/cards/194751/images/790c3051d631b61c23f6eb36b4134932.jpg');
const filePath2 = path.join(__dirname, '../test_data/cards/194751/images/b9d86427943d5b271d8053c1dd2796bc.jpg');
const imgData1 = fs.readFileSync(filePath1);
const imgData2 = fs.readFileSync(filePath2);

const value = 1;

const img1 = new ImageData(imgData1);
const img2 = new ImageData(imgData2);
const input = new ImageInput(value,[img1,img2]);

// 将字节数组传递给 Wasm 模块
test_image(input);