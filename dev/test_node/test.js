const fs = require('fs');
const path = require('path');
const { initialize, inference } = require('..\\..\\pkg\\card_recognize.js');

// 构造文件路径（跨平台）
const filePath = path.join(__dirname, '../test_data/cards/194751/scan.json');
const image1_path = 'D:\\workspace\\github\\card_recognize\\dev\\test_data\\cards\\194751\\images\\790c3051d631b61c23f6eb36b4134932.jpg';
const image2_path = 'D:\\workspace\\github\\card_recognize\\dev\\test_data\\cards\\194751\\images\\b9d86427943d5b271d8053c1dd2796bc.jpg';

// 读取 JSON 文件
fs.readFile(filePath, 'utf8', (err, data) => {
  if (err) {
    console.error('Error reading file:', err);
    return;
  }
  
  // 将读取的 JSON 数据转换为字符串
  const jsonString = JSON.stringify(JSON.parse(data));

  initialize(jsonString);

  const inputImage = {
    task_id: "123456",
    images: [image1_path.toString(),image2_path.toString()],
    calling_type: 0
  };
  
  const jsonString_iamge = JSON.stringify(inputImage);

  const result = inference(jsonString_iamge);
  
  console.log(result); 
});