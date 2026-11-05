const fs = require('fs');
const path = require('path');
const logFile = 'C:\\Users\\grimm\\.local\\share\\opencode\\shell\\38cb6929c4c5ec7a1c7862af83c253400ee50d97\\sh_1ab2407b80012x6ou0G0S1SqGu.out';
const testOutput = 'C:\\Users\\grimm\\Desktop\\Dev\\UniPanel\\test_output.log';

let iterations = 0;

function check() {
  try {
    // Check both files
    const shellOut = fs.existsSync(logFile) ? fs.readFileSync(logFile, 'utf8') : '';
    const testOut = fs.existsSync(testOutput) ? fs.readFileSync(testOutput, 'utf8') : '';
    const combined = shellOut + testOut;
    
    iterations++;
    
    if (combined.includes('test result:')) {
      console.log('=== TESTS COMPLETE ===');
      combined.split('\n').filter(l => l.includes('test result:')).forEach(l => console.log(l.trim()));
      process.exit(0);
    }
    if (iterations % 3 === 0) {
      const compiles = combined.split('\n').filter(l => l.includes('Compiling')).slice(-5);
      console.log(`[${iterations}] ${compiles.join(' | ')}`);
    }
    setTimeout(check, 30000);
  } catch (e) {
    console.log('Error:', e.message);
    setTimeout(check, 15000);
  }
}

check();
