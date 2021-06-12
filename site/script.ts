import { Engine } from "../wasm/frontend/dash";
import md5 from "md5";

declare var EmbeddedConsole: any;
declare var CodeMirror: any;

const CHECKSUM = "9914b86dbbbb5d01f801f1da36449021";
const WASM_URL = "http://dash.y21_.repl.co/wasm/v1";

const url = new URL(document.location.href);

const engine = new Engine();
(async () => {
    const buffer = await fetch(WASM_URL).then(x => x.arrayBuffer());

    // We load the WebAssembly blob from another website and make sure the checksum matches for security reasons
    // So the host doesn't serve a malicious/invalid blob
    const u8 = new Uint8Array(buffer);
    const originChecksum = md5(u8);
    if (originChecksum !== CHECKSUM) {
        throw new Error(`Checksum doesn't match. (${originChecksum} != ${CHECKSUM})`);
    }

    await engine.init(buffer);
})();

const editorElement = document.getElementById("code");
const consoleElement = document.getElementById("console");
const executeBtn = document.getElementById("execute-btn");
const shareBtn = document.getElementById("save");
const editor = CodeMirror.fromTextArea(editorElement, {
    lineNumbers: true,
    mode: "javascript",
    theme: "ambiance",
    extraKeys: {
        "Ctrl-Space": "autocomplete"
    }
});

{
    const maybeCode = url.searchParams.get("code");
    if (maybeCode) {
        const code = atob(maybeCode);
        editor.setValue(code);
    }
}

const ec = new EmbeddedConsole(consoleElement);
ec.log("Welcome to the dash playground");
ec.log("Write JavaScript code in the box above and press \"Execute code\" to run it.");
ec.log("Note that this engine is still a very early work in progress. See progress here: https://github.com/y21/dash/blob/master/progress.md");

executeBtn!.addEventListener("click", () => {
    if (!engine.initialized) {
        ec.warn("Cannot use engine before initialization. Please wait a few seconds.");
        return;
    }

    try {
        const value = editor.getValue();
        const result = engine.eval(value);

        const maybeNumber = Number(result);

        if (!isNaN(maybeNumber)) {
            ec.log(maybeNumber);
        } else {
            ec.log(result);
        }
    } catch(e) {
        ec.error(e);
    }
});

shareBtn!.addEventListener("click", () => {
    const value = editor.getValue();
    const encoded = btoa(value);
    url.searchParams.set("code", encoded);
    document.location.href = url.toString();
});