const fs = require('fs');
const content = fs.readFileSync('src-tauri/tests/challenger_utf8_truncation_test.rs', 'utf8');
const lines = content.split(/\r?\n/);
lines.forEach((line, idx) => {
  if (line.includes('let emojis =')) {
    console.log('Line', idx + 1, line);
    const m = line.match(/\"([^\"]+)\"/);
    if (m) {
      const str = m[1];
      console.log('Total byte length:', Buffer.from(str, 'utf8').length);
      console.log('Chars count:', [...str].length);
      for (const ch of str) {
        const cp = ch.codePointAt(0);
        console.log(ch, 'codePoint: 0x' + cp.toString(16), 'bytes:', Buffer.from(ch, 'utf8').length);
      }
    }
  }
});
