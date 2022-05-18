const is = {
    string: (value) => typeof value === 'string',
    number: (value) => typeof value === 'number',
    boolean: (value) => typeof value === 'boolean',
    nullish: (value) => value === null || value === undefined,
    error: (value) => value instanceof Error,
    array: (value) => value instanceof Array, // TODO: Array.isArray
    function: (value) => typeof value === 'function',
    looseObject: function (value) {
        return !this.nullish(value) && typeof value === 'object';
    },
    strictObject: function (value) {
        // TODO: use Array.isArray once we have it
        return this.looseObject(value) && !(value instanceof Array);
    }
};

function inner(value, indentation) {
    if (is.string(value)) {
        return value;
    }

    if (is.error(value)) {
        return value.stack;
    }

    if (is.strictObject(value)) {
        const keys = Object.keys(value);
        const hasElements = keys.length > 0;

        let repr;
        if (value.constructor !== Object) {
            repr = value.constructor.name + ' {';
        } else {
            repr = '{';
        }

        if (hasElements) repr += ' ';

        for (let i = 0; i < keys.length; i++) {
            if (i > 0) {
                repr += ', ';
            }

            const key = keys[i];
            repr += key + ': ' + inner(value[key]);
        }

        if (hasElements) repr += ' ';

        repr += '}';

        return repr;
    }

    if (is.array(value)) {
        const len = value.length;

        let repr = '[';

        for (let i = 0; i < len; i++) {
            if (i > 0) {
                repr += ', ';
            }

            repr += inner(value[i], indentation);
        }

        repr += ']';
        return repr;
    }

    if (is.function(value)) {
        const name = value.name || '(anonymous)';

        return '[Function: ' + name + ']';
    }

    // if nothing matched, stringify
    return String(value);
}

export default function (value) {
    return inner(value, 0);
}
