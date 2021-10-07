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
 * Size of a pointer in bytes
 */
const POINTER_SIZE = 1 << 2;

/**
 * Exported Rust functions that can be called from JavaScript
 */
interface Exports {
    memory: WebAssembly.Memory,
    inspect_create_vm_error: (ptr: Pointer) => Pointer,
    alloc: (size: Usize) => Pointer,
    dealloc: (ptr: Pointer, len: Usize) => void,
    free_c_string: (ptr: Pointer) => void,
    eval: (ptr: Pointer) => Pointer,
    create_vm: () => Pointer,
    create_vm_from_string: (ptr: Pointer) => Pointer,
    vm_interpret: (ptr: Pointer) => Pointer,
    vm_eval: (vm_ptr: Pointer, source_ptr: Pointer) => Pointer,
    vm_set_gc_object_threshold: (ptr: Pointer, threshold: Usize) => void,
    vm_run_async_tasks: (ptr: Pointer) => void,
    value_inspect: (ptr: Pointer) => Pointer,
    value_to_string: (ptr: Pointer) => Pointer,
    free_vm: (ptr: Pointer) => void,
    free_create_vm_from_string_result: (ptr: Pointer) => void,
    free_vm_interpret_result: (ptr: Pointer) => void,
    free_eval_result: (ptr: Pointer) => void,
    free_vm_eval: (ptr: Pointer) => void,
    version(): Pointer,
    __data_end: WebAssembly.Global,
    __heap_base: WebAssembly.Global,
}

namespace errors {
    /**
     * Thrown when attempting to access a resource that has been freed
     */
    export const RESOURCE_FREED = new Error("Resource has been freed");
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

abstract class TaggedUnion<D> {
    protected ptr: Pointer;
    protected internal: Internal;

    constructor(internal: Internal, ptr: Pointer) {
        this.ptr = ptr;
        this.internal = internal;
    }

    protected rawDiscriminant(): number {
        // First byte is the discriminant
        return this.internal.withDataView((view) => view.getUint8(this.ptr));
    }

    abstract discriminant(): D;

    getDataPointer(): number {
        return this.ptr + ENUM_DATA_OFFSET;
    }
}

/**
 * Wrapper for a Result coming from WebAssembly
 */
class Result extends TaggedUnion<ResultDiscriminant> {
    constructor(internal: Internal, ptr: Pointer) {
        super(internal, ptr);
    }

    /**
     * The discriminant of this Result
     * @returns {ResultDiscriminant}
     */
    discriminant(): ResultDiscriminant {
        return this.rawDiscriminant() === 0
            ? ResultDiscriminant.OK
            : ResultDiscriminant.ERR;
    }

    /**
     * Whether this Result is Ok
     * @returns {boolean}
     */
    isOk(): boolean {
        return this.discriminant() === ResultDiscriminant.OK;
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
class Option extends TaggedUnion<OptionDiscriminant> {

    constructor(internal: Internal, ptr: Pointer) {
        super(internal, ptr);
    }

    /**
     * The discriminant of this Option
     * @returns {OptionDiscriminant}
     */
    discriminant(): OptionDiscriminant {
        return this.rawDiscriminant() === 0
            ? OptionDiscriminant.SOME
            : OptionDiscriminant.NONE;
    }

    /**
     * Whether this Option contains a value
     * @returns {boolean}
     */
    isSome(): boolean {
        return this.discriminant() === OptionDiscriminant.SOME;
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

export class VM {
    private internal: Internal;
    private ptr: Pointer;
    private freed: boolean;

    constructor(internal: Internal, ptr: Pointer) {
        this.internal = internal;
        this.ptr = ptr;
        this.freed = false;
    }

    /**
     * Frees memory that belongs to this VM
     */
    public free() {
        if (this.freed) throw errors.RESOURCE_FREED;
        this.internal.wasm.free_vm(this.ptr);
        this.freed = true;
    }

    /**
     * Sets the threshold for number of objects needed before the garbage collector runs
     * Note: the VM will automatically adjust this value to what is appropriate for the current number of objects
     * @param {number} threshold 
     */
    public setGcObjectThreshold(threshold: number) {
        if (this.freed) throw errors.RESOURCE_FREED;
        this.internal.wasm.vm_set_gc_object_threshold(this.ptr, threshold);
    }

    /**
     * Runs scheduled async tasks
     */
    public runAsyncTasks() {
        if (this.freed) throw errors.RESOURCE_FREED;
        this.internal.wasm.vm_run_async_tasks(this.ptr);
    }

    /**
     * Evaluates the given source code and returns the result as a string.
     * 
     * @param {string} source
     * @returns {string | undefined}
     */
    public eval(source: string) {
        if (this.freed) throw errors.RESOURCE_FREED;

        const sourcePtr = this.internal.writeString(source);
        const resultPtr = this.internal.wasm.vm_eval(this.ptr, sourcePtr);

        try {
            const result = new Result(this.internal, resultPtr)
                .unwrap(this.internal.wasm.inspect_create_vm_error.bind(this.internal));

            const option = new Option(this.internal, result);
            if (!option.isSome()) return;

            const value = option.getDataPointer();

            const valuePtr = this.internal.withDataView((view) => view.getUint32(value, true));

            const inspectPtr = this.internal.wasm.value_inspect(valuePtr);
            const message = this.internal.readString(inspectPtr);
            this.internal.wasm.free_c_string(inspectPtr);

            return message;
        } finally {
            this.internal.wasm.free_c_string(sourcePtr);
            this.internal.wasm.free_vm_eval(resultPtr);
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
     * Returns the version of the engine
     */
    public getVersion() {
        const ptr = this.getInternal().wasm.version();
        const version = this.getInternal().readString(ptr);
        this.getInternal().wasm.free_c_string(ptr);
        return version;
    }

    /**
     * Creates a VM.
     * You must manually free memory allocated for this VM by calling free on the returned object.
     */
    public createVM() {
        const internal = this.getInternal();
        const vmPointer = internal.wasm.create_vm();
        const vm = new VM(internal, vmPointer);
        // TODO: FinalizationRegistry.register(vm)

        return vm;
    }

    /**
     * Convenient method for evaluating a JavaScript source string and returning the last value.
     * @param {string} source 
     * @returns 
     */
    public eval(source: string) {
        const internal = this.getInternal();
        const stringPtr = internal.writeString(source);
        const evalPtr = internal.wasm.eval(stringPtr);
        let vmPtr: Pointer = 0;

        try {
            const result = new Result(internal, evalPtr).unwrap(internal.wasm.inspect_create_vm_error.bind(internal));

            vmPtr = internal.withDataView((view) => view.getUint32(evalPtr + 12, true));

            const maybeValue = new Option(internal, result);
            if (!maybeValue.isSome()) return;
            const value = maybeValue.getDataPointer();

            const valuePtr = internal.withDataView((view) => view.getUint32(value, true));

            const inspectPtr = internal.wasm.value_inspect(valuePtr);
            const message = internal.readString(inspectPtr);
            internal.wasm.free_c_string(inspectPtr);

            return message;
        } finally {
            internal.wasm.free_c_string(stringPtr);
            internal.wasm.free_eval_result(evalPtr);

            if (vmPtr) {
                internal.wasm.free_vm(vmPtr);
            }
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