const fs = require('fs');

// 读取指定的JSON文件
const filePath = './pkg/package.json';
fs.readFile(filePath, 'utf8', (err, data) => {
  if (err) {
    console.error(err);
    return;
  }

  // 解析JSON数据
  let jsonData = JSON.parse(data);

  // 修改变量的值
  jsonData.name = '@suilang/rrule-rust';

  // 将修改后的内容写回文件
  fs.writeFile(filePath, JSON.stringify(jsonData, null, 2), 'utf8', (err) => {
    if (err) {
      console.error(err);
      return;
    }
    console.log('JSON文件已成功修改并写回。');
  });
});
