const fs = require('fs');

// 读取图像文件
fs.readFile('../test_data/cards/194751/images/790c3051d631b61c23f6eb36b4134932.jpg', (err, data) => {
    if (err) {
        console.error('Error reading image file:', err);
        return;
    }

    // 将图像数据转换为 base64 字符串
    const base64Image = Buffer.from(data).toString('base64');

    // base64Image 就是转换后的 base64 字符串
    console.log(base64Image);
});