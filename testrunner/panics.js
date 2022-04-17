const filename = process.argv[2];
const fs = require('fs');
const file = fs.readFileSync(filename, 'utf8');
const lines = file.split(/\r?\n/g)
    .filter(x => x.startsWith('thread \'main\' panicked at'));

for (const line of lines) console.log(line);
