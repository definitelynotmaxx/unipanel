const fs = require('fs');
const path = require('path');
const logFile = path.join(
  process.env.LOCALAPPDATA || process.env.APPDATA || 'C:\\Users\\grimm\\.local\\share\\opencode\\shell',
  '38cb6929c4c5ec7a1c7862af83c253400ee50d97',
  'sh_1ab1520a3001ZIGFkaIsVyyw0B.out'
);

let iteration = 0;

function check() {
  try {
    const content = fs.readFileSync(logFile, 'utf8');
    iteration++;
    
    if (content.includes('test result:')) {
      const lines = content.split('\n').filter(l => l.includes('test result:'));
      console.log('=== TESTS COMPLETE ===');
      lines.forEach(l => console.log(l.trim()));
      process.exit(0);
    }
    if (iteration % 5 === 0) {
      const lastLines = content.split('\n').filter(l => l.includes('Compiling')).slice(-5);
      console.log(`Check ${iteration}: ${lastLines.join(', ')}`);
    }
    if (iteration >= 100) {
      console.log('Timeout - still building');
      process.exit(1);
    }
    setTimeout(check, 60000);
  } catch (e) {
    console.log('Error:', e.message);
    setTimeout(check, 30000);
  }
}

check();
