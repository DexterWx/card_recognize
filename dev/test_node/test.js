const fs = require('fs');
const path = require('path');
const { initialize, inference } = require('../../pkg/card_recognize.js');

// 构造文件路径（跨平台）
const filePath = path.join(__dirname, '../test_data/cards/194751/scan.json');
// 读取图像文件
const filePath1 = path.join(__dirname, '../test_data/cards/194751/images/790c3051d631b61c23f6eb36b4134932.jpg');
const filePath2 = path.join(__dirname, '../test_data/cards/194751/images/b9d86427943d5b271d8053c1dd2796bc.jpg');
const imgData1 = fs.readFileSync(filePath1);
const imgData2 = fs.readFileSync(filePath2);
// 将图像数据转换为 base64 字符串
const base64Image1 = 'data:image/jpeg;base64,' + imgData1.toString('base64');
const base64Image2 = 'data:image/jpeg;base64,' + imgData2.toString('base64');

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
    images: [base64Image1, base64Image2],
    calling_type: 0
  };
  
  const jsonString_iamge = JSON.stringify(inputImage);
  console.time('myTimer');
  const result = inference(jsonString_iamge);
  console.log(result);
  console.timeEnd('myTimer');
  
  // console.log(result); 
});