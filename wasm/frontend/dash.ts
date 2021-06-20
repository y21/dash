/**
 * A value that can resolve to an ArrayBuffer
 */
type BinaryResolvable = string | Uint8Array | ArrayBuffer;
/**
 * Pointer
 */
type Pointer = number;
/**
 * Pointer-sized number
 */
type Usize = number;
/**
 * 64-bit pointer
 */
type Pointer64 = BigInt;

/**
 * Data offset for Rust enums
 */
const ENUM_DATA_OFFSET = 1 << 2;

/**
 * Exported Rust functions that can be called from JavaScript
 */
interface Exports {
    memory: WebAssembly.Memory,
    alloc: (size: Usize) => Pointer,
    dealloc: (ptr: Pointer, len: Usize) => void,
    __data_end: WebAssembly.Global,
    __heap_base: WebAssembly.Global,
    create_vm: (ptr: Pointer) => Pointer,
    eval: (ptr: Pointer) => Pointer,
    free_c_string: (ptr: Pointer) => void,
    free_create_vm_result: (ptr: Pointer) => void,
    free_eval_result: (ptr: Pointer) => void,
    free_vm_interpret_result: (ptr: Pointer) => void,
    inspect_create_vm_error: (ptr: Pointer) => Pointer,
    inspect_vm_interpret_error: (ptr: Pointer) => Pointer,
    value_inspect: (ptr: Pointer) => Pointer,
    value_to_string: (ptr: Pointer) => Pointer,
    vm_interpret: (ptr: Pointer) => Pointer
}

/**
 * Enum discriminator for Rust Result
 */
enum ResultDiscriminant {
    OK = 0,
    ERR = 1
}

/**
 * Enum discriminator for Rust Option
 */
enum OptionDiscriminant {
    SOME = 0,
    NONE = 1
}

/**
 * Wrapper for a Result coming from WebAssembly
 */
class Result {
    private ptr: Pointer;
    private internal: Internal;

    constructor(internal: Internal, ptr: Pointer) {
        this.internal = internal;
        this.ptr = ptr;
    }

    /**
     * The discriminant of this Result
     * @returns {ResultDiscriminant}
     */
    discriminant(): ResultDiscriminant {
        return this.internal.withDataView((view) => view.getUint8(this.ptr) === 0 ? ResultDiscriminant.OK : ResultDiscriminant.ERR);
    }

    /**
     * Whether this Result is Ok
     * @returns {boolean}
     */
    isOk(): boolean {
        return this.discriminant() === ResultDiscriminant.OK;
    }

    /**
     * A pointer to data to this enum
     * @returns {Pointer}
     */
    getDataPointer(): Pointer {
        return this.ptr + ENUM_DATA_OFFSET;
    }

    /**
     * "Unwraps" a Result - returns the data pointer or throws an error
     * It calls the provided function and expects it to return a pointer to a CString if Err
     * @param {(ptr: Pointer) => Pointer}
     * @returns {Pointer}
     */
    unwrap(failInspect: (ptr: Pointer) => Pointer): Pointer {
        const isOk = this.isOk();
        const dataPtr = this.getDataPointer();
        if (!isOk) {
            const inspected = failInspect(dataPtr);
            const message = this.internal.readString(inspected);
            this.internal.wasm.free_c_string(inspected);
            throw new Error(message);
        }
        return dataPtr;
    }
}

/**
 * Wrapper for an Option coming from WebAssembly
 */
class Option {
    private ptr: Pointer;
    private internal: Internal;

    constructor(internal: Internal, ptr: Pointer) {
        this.internal = internal;
        this.ptr = ptr;
    }

    /**
     * The discriminant of this Option
     * @returns {OptionDiscriminant}
     */
    discriminant(): OptionDiscriminant {
        return this.internal.withDataView((view) => view.getUint8(this.ptr) === 0 ? OptionDiscriminant.SOME : OptionDiscriminant.NONE);
    }

    /**
     * Whether this Option contains a value
     * @returns {boolean}
     */
    isSome(): boolean {
        return this.discriminant() === OptionDiscriminant.SOME;
    }

    /**
     * A pointer to data to contained data
     * @returns {Pointer}
     */
    getDataPointer(): Pointer {
        return this.ptr + ENUM_DATA_OFFSET;
    }
}

/**
 * Dash internals
 * This is not meant to be used by end users. It operates directly on the WebAssembly binary
 */
class Internal {
    public wasm: Exports;
    constructor(wasm: Exports) {
        this.wasm = wasm;
    }

    /**
     * Creates a DataView to WebAssembly memory
     * @returns {DataView}
     */
    public getDataView(): DataView {
        // TODO: cache dataview
        return new DataView(this.wasm.memory.buffer);
    }

    /**
     * Calls the given function with a new dataview as parameter
     * @param {(dv: DataView) => T} fn - Callback function
     */
    public withDataView<T>(fn: (dv: DataView) => T): T {
        const dv = this.getDataView();
        return fn(dv);
    }

    /**
     * Allocates a chunk with n bytes of memory.
     * The caller must deallocate memory manually.
     * @param {number} bytes - Number of bytes to allocate
     */
    public alloc(bytes: number): Pointer {
        const ptr = this.wasm.alloc(bytes);
        assert(ptr !== 0);
        return ptr;
    }

    /**
     * Allocates memory for a string, fills it with characters and returns a pointer to it
     * The string must not have null bytes in it (violates C-String requirements)
     * @param {string} source - The string to write
     * @returns {Pointer}
     */
    public writeString(source: string): Pointer {
        const ptr = this.alloc(source.length + 1);
        this.withDataView((view) => {
            for (let i = 0; i < source.length; ++i) {
                const cur = source.charCodeAt(i);
                if (cur === 0) throw new Error("String cannot contain null characters");

                view.setUint8(ptr + i, cur);
            }
            view.setUint8(ptr + source.length, 0);
        });
        return ptr;
    }

    /**
     * Reads a string from WebAssembly memory. The string must be a CString, meaning it must have a null byte at the end.
     * If the string does not end in \0, behavior is undefined and this operation *can* lead to reading memory that doesn't belong to allocated chunk.
     * @param {Pointer} ptr - A pointer to the (start of the) string
     * @returns {string}
     */
    public readString(ptr: Pointer): string {
        return this.withDataView((view) => {
            let s = "";
            for (let i = 0; ; ++i) {
                const cur = view.getUint8(ptr + i);
                if (cur === 0) break;
                s += String.fromCharCode(cur);
            }
            return s;
        });
    }
}

class VM {
    private internal: Internal;
    private vmDataPtr: Pointer;
    private sourcePtr: Pointer;
    private sourceLen: Usize;
    private _freed: boolean;

    constructor(internal: Internal, vmDataPtr: Pointer, sourcePtr: Pointer, sourceLen: Usize) {
        this.internal = internal;
        this.vmDataPtr = vmDataPtr;
        this.sourcePtr = sourcePtr;
        this.sourceLen = sourceLen;
        this._freed = false;
    }

    /**
     * Frees memory that belongs to this VM
     * This must be called when a VM is manually created, e.g. by calling `engine.createVM()`, otherwise memory is leaked
     */
    free() {
        if (this._freed) throw new Error("This resource has been freed up already");
        this.internal.wasm.dealloc(this.sourcePtr, this.sourceLen);
        const vmResultPtr = this.vmDataPtr - ENUM_DATA_OFFSET;
        this.internal.wasm.free_create_vm_result(vmResultPtr);
        this._freed = true;
    }

    /**
     * Executes bytecode associated to this VM, inspects the last value and returns it as a string
     * @returns {string | undefined}
     */
    exec(): string | undefined {
        const resultPtr = this.internal.wasm.vm_interpret(this.vmDataPtr);
        const valueResult = new Result(this.internal, resultPtr);

        try {
            const valueResultPtr = valueResult.unwrap(this.internal.wasm.inspect_vm_interpret_error.bind(this.internal));

            const valueOption = new Option(this.internal, valueResultPtr);
            if (!valueOption.isSome()) return;

            const valueInspectPtr = this.internal.wasm.value_inspect(valueOption.getDataPointer());
            const message = this.internal.readString(valueInspectPtr);
            this.internal.wasm.free_c_string(valueInspectPtr);
            return message;
        } finally {
            this.internal.wasm.free_vm_interpret_result(resultPtr);
        }
    }
}

export class Engine {
    private internal?: Internal;
    constructor() { }

    /**
     * Initializes this Engine by compiling the WebAssembly binary
     * @param {BinaryResolvable} binary 
     */
    async init(binary: BinaryResolvable) {
        const buffer = await resolveBinary(binary);

        const wasm = await WebAssembly.instantiate(buffer);
        this.internal = new Internal(wasm.instance.exports as any);
    }

    /**
     * Whether this Engine is initialized
     */
    public get initialized() {
        return Boolean(this.internal);
    }

    /**
     * Returns this.internal, or throws an error if it's not set (presumably because it hasn't been initialized yet)
     */
    private getInternal(): Internal {
        if (this.internal) return this.internal;
        throw new Error("This Engine has not been initialized yet. " +
            "Call init on this instance first and wait for its promise to resolve");
    }

    /**
     * Creates a VM. You must manually free memory allocated for this VM by calling free on the returned object.
     * @param {string} source 
     * @returns 
     */
    public createVM(source: string) {
        const internal = this.getInternal();
        const stringPtr = internal.writeString(source);
        const vmResultPtr = internal.wasm.create_vm(stringPtr);
        const vmResult = new Result(internal, vmResultPtr);

        try {
            const vmDataPtr = vmResult.unwrap(internal.wasm.inspect_create_vm_error.bind(internal));
            return new VM(internal, vmDataPtr, stringPtr, source.length + 1);
        } catch (e) {
            internal.wasm.dealloc(stringPtr, source.length + 1);
            internal.wasm.free_create_vm_result(vmResultPtr);
            throw e;
        }
    }

    /**
     * Convenient method for evaluating a JavaScript source string and returning the last value.
     * @param {string} source 
     * @returns 
     */
    public eval(source: string) {
        const internal = this.getInternal();
        const stringPtr = internal.writeString(source);
        const resultPtr = internal.wasm.eval(stringPtr);
        const result = new Result(internal, resultPtr);

        try {
            const valueOption = new Option(internal, result.unwrap(internal.wasm.inspect_create_vm_error));

            if (!valueOption.isSome()) return;

            const valueInspectPtr = internal.wasm.value_inspect(valueOption.getDataPointer());
            const message = internal.readString(valueInspectPtr);
            internal.wasm.free_c_string(valueInspectPtr);
            return message;
        } finally {
            internal.wasm.free_eval_result(resultPtr);
            internal.wasm.dealloc(stringPtr, source.length + 1);
        }
    }
}

/**
 * Tries to resolve a binary given BinaryResolvable
 * @param {BinaryResolvable} binary - A parameter that can resolve to a binary
 * @returns {Promise<ArrayBuffer>}
 */
async function resolveBinary(binary: BinaryResolvable): Promise<ArrayBuffer> {
    if (binary instanceof ArrayBuffer) return binary;
    else if (binary instanceof Uint8Array) return binary.buffer;
    else return fetch(binary).then(x => x.arrayBuffer());
}

/**
 * Asserts a condition. This is used to ensure a condition is never false, and unrecoverable errors
 */
function assert<T>(cond: T): T {
    if (!cond) throw new Error("Assertion failed: " + cond);
    return cond;
}