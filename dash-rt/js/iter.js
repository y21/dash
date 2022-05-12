const iteratorProto = {
    map: function (cb) {
        const o = Object.create(this);
        const _this = this;

        o[Symbol.iterator] = function* () {
            const it = _this[Symbol.iterator]();

            let item;
            while (!(item = it.next()).done) {
                yield cb(item.value);
            }
        };
        return o;
    },

    forEach: function (cb) {
        const it = this[Symbol.iterator]();

        let item;
        while (!(item = it.next()).done) {
            cb(item.value);
        }
    },

    sum: function () {
        let sum = 0;
        this.forEach((i) => sum += i);

        // TODO: get rid of this hack
        // local values remain boxed even after returning
        return sum + 0;
    }
};

function from(iterable) {
    if (!iterable[Symbol.iterator]) {
        throw new Error('Provided value is missing the @@iterator field');
    }

    const obj = Object.create(iteratorProto);
    obj[Symbol.iterator] = function* () {
        const it = iterable[Symbol.iterator]();

        let item;
        while (!(item = it.next()).done) {
            yield item.value;
        }
    }
    return obj;
}

export default from;
