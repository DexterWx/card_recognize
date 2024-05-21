const fs = require('fs');
const path = require('path');
const { initialize, inferenceSecond } = require('../../index.js');

// 构造文件路径（跨平台）
const filePath = path.join(__dirname, '../test_data/cards/194751/scan_second.json');

// 读取 JSON 文件
fs.readFile(filePath, 'utf8', (err, data) => {
  if (err) {
    console.error('Error reading file:', err);
    return;
  }
  
  // 将读取的 JSON 数据转换为字符串
  const jsonString = JSON.stringify(JSON.parse(data));
  console.time('myTimer');
  const result = inferenceSecond(jsonString);
  console.timeEnd('myTimer');
});