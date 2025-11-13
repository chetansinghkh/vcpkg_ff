const ffmpeg = require('./build/Release/ffmpeg_node.node');
const fs = require('fs');

console.log('开始生成测试视频...\n');

// 方案1: 生成带音频的测试视频（使用两个 lavfi 输入）
console.log('方案1: 生成带音频的测试视频...');
const result1 = ffmpeg.run([
    '-f', 'lavfi',
    '-i', 'testsrc=duration=5:size=640x480:rate=30',
    '-f', 'lavfi',
    '-i', 'sine=frequency=1000:duration=5',
    '-c:a', 'aac',
    '-shortest',
    '-y',
    'test_with_audio.mp4'
]);
console.log(result1)