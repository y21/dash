const fs = require("fs");

function load(path) {
    const arr = fs.readFileSync(path, "utf8").split(/\r?\n/);
    return new Set(arr.filter(c => c.endsWith("did not pass")));
}

const old = load(process.argv[2]);
const ne = load(process.argv[3]);

for (const entry of ne.keys()) {
    if (!old.has(entry)) {
        console.log(entry);
    }
}
