const fs = require('fs');
const c = fs.readFileSync('C:/Users/grimm/.local/share/opencode/shell/38cb6929c4c5ec7a1c7862af83c253400ee50d97/sh_1ab1520a3001ZIGFkaIsVyyw0B.out', 'utf8');
if (c.includes('test result')) {
  console.log('TESTS_DONE');
  c.split('\n').filter(l => l.includes('test result:')).forEach(l => console.log(l.trim()));
} else if (c.includes('Finished')) {
  console.log('BUILD_DONE');
} else {
  const compiles = c.split('\n').filter(l => l.includes('Compiling')).slice(-5);
  console.log('DEPS:', compiles.join(' | '));
}
