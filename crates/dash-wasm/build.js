const cp = require('child_process');
const fs = require('fs');

cp.execSync('wasm-pack build --target no-modules');
let file = fs.readFileSync('pkg/wasm.js', 'utf8');
file = file.replace('async function init(input)', 'async function init()');
file = file.replace(/if \(typeof input === 'undefined'(.+\s*){9}/, '');
file = file.replace(/if \(typeof input === 'string'(.+\s*){3}/, 'const input = require(\'fs/promises\').readFile(\'pkg/wasm_bg.wasm\');\n');
file = file.replace(/(wasm_bindgen = Object.+)/, '$1\nmodule.exports = wasm_bindgen;');
fs.writeFileSync('pkg/wasm.js', file);
