const fs = require('fs');

const [, , pathBefore, pathAfter] = process.argv;

if (!pathBefore || !pathAfter) throw new Error('Paths are missing');

const R = /Some\("([^"]+)/g;
const before = new Set(Array.from(fs.readFileSync(pathBefore, 'utf-8').matchAll(R)).map(x => x[1]));
const now = Array.from(fs.readFileSync(pathAfter, 'utf-8').matchAll(R)).map(x => x[1]);

for (const test of now) {
    if (!before.has(test)) {
        console.log(test)
    }
}
