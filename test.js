const ffmpeg = require('./build/Release/ffmpeg_node.node');
const fs = require('fs');


//将input.mp4转换为m3u8
const result = ffmpeg.run([
    '-i', 'input.mp4',
    '-c:v', 'libx264',
    '-c:a', 'aac',
    '-hls_time', '10',
    '-hls_list_size', '0',
    'output.m3u8'
]);
console.log(result)
