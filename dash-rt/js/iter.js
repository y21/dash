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

    zip: function (iterable) {
        const o = Object.create(this);
        const _this = this;

        o[Symbol.iterator] = function* () {
            const it1 = _this[Symbol.iterator]();
            const it2 = iterable[Symbol.iterator]();

            let item1;
            let item2;
            while (
                !(item1 = it1.next()).done &&
                !(item2 = it2.next()).done
            ) {
                yield [item1.value, item2.value];
            }
        }

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
        return sum;
    },

    min: function () {
        let min = null;
        this.forEach((i) => {
            if (min === null || i < min) {
                min = i;
            }
        });
        return min;
    },

    max: function () {
        let max = null;
        this.forEach((i) => {
            if (max === null || i > max) {
                max = i;
            }
        });
        return max;
    },

    toArray: function () {
        let x = [];
        this.forEach((i) => x.push(i));
        return x;
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
