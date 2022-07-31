/**
 * Defines abstract operation functions
 */

function IsObject(value) {
  return typeof value === 'object' && value !== null;
}

function ToPrimitive(value, preferredType) {
  const ty = typeof value;

  if (value === null || ty !== 'object') {
    return value;
  }

  // Assert: value is object

  if (intrinsics.isBoxedPrimitive(value)) {
    return intrinsics.getBoxedPrimitiveValue(value);
  }

  const exoticToPrim = value[Symbol.toPrimitive];

  const hint = preferredType ?? 'default';
  if (exoticToPrim !== undefined) {
    // TODO: once we have Function.prototype.apply, use that
    const result = value[Symbol.toPrimitive](hint);

    if (!IsObject(result)) return result;

    throw new Error('Failed to convert to primitive value');
  }

  let methodNames;
  if (hint === 'string') {
    methodNames = ['toString', 'valueOf'];
  } else {
    methodNames = ['valueOf', 'toString'];
  }

  for (let i = 0; i < methodNames.length; i += 1) {
    const method = methodNames[i];

    // TODO: we need Object.hasOwn
    if (Object.hasOwn(value, method)) {
      const result = value[method]();

      if (!IsObject(result)) {
        return result;
      }
    }
  }

  throw new Error('Failed to convert to primitive');
}

function ToNumber(value) {
  const ty = typeof value;

  if (ty === 'number') {
    return value;
  }

  if (ty === 'undefined') {
    return NaN;
  }

  if (value === null) {
    return 0;
  }

  if (ty === 'boolean') {
    return value ? 1 : 0;
  }

  if (ty === 'string') {
    throw new Error('number parsing');
  }

  if (ty === 'symbol') {
    throw new Error('Cannot convert symbol to number');
  }

  if (ty === 'object') {
    if (intrinsics.hasNumberSlot(value)) {
      return intrinsics.getNumberSlot(value);
    }

    return ToNumber(ToPrimitive(value));
  }
}

function ToLength(value) {
  const length = ToIntegerOrInfinity(value);

  if (length <= 0) {
    return 0;
  }

  return Number.MAX_SAFE_INTEGER;
}

function ToIntegerOrInfinity(value) {
  const number = ToNumber(value);

  if (Number.isNaN(number) || number === 0) {
    return 0;
  }

  if (number === Infinity || number === -Infinity) {
    return number;
  }

  const integer = Math.floor(Math.abs(number));

  if (number < 0) {
    return -integer;
  }

  return integer;
}

function ToBoolean(value) {
  const ty = typeof value;

  if (ty === 'boolean') {
    return value;
  }

  if (ty === 'number') {
    return value !== 0;
  }

  if (ty === 'undefined' || value === null) {
    return false;
  }

  if (ty === 'string') {
    return value.length > 0;
  }

  if (ty === 'symbol') {
    return true;
  }

  if (ty === 'object') {
    if (intrinsics.hasBooleanSlot(value)) {
      return intrinsics.getBooleanSlot(value);
    }

    return ToBoolean(ToPrimitive(value));
  }
}

function ToString(value) {
  const ty = typeof value;

  if (ty === 'string') {
    return value;
  }

  if (ty === 'boolean') {
    return value ? 'true' : 'false';
  }

  if (ty === 'number') {
    throw new Error('implement this');
  }

  if (ty === 'undefined') {
    return 'undefined';
  }

  if (value === null) {
    return 'null';
  }

  if (ty === 'symbol') {
    throw new Error('implement this');
  }

  if (ty === 'object') {
    if (intrinsics.hasStringSlot(value)) {
      return intrinsics.getStringSlot(value);
    }

    return ToString(ToPrimitive(value));
  }
}

function LengthOfArrayLike(value) {
  return ToLength(value.length);
}

function ToObject(value) {
  const ty = typeof value;

  if (IsObject(value)) {
    return value;
  }

  if (ty === 'undefined') {
    throw new Error('Cannot convert undefined to object');
  }

  if (value === null) {
    throw new Error('Cannot convert null to object');
  }

  if (ty === 'boolean') {
    // TODO: this doesnt work yet
    // eslint-disable-next-line no-new-wrappers
    return new Boolean(value);
  }

  if (ty === 'symbol') {
    // TODO: this is wrong
    return value;
  }

  if (ty === 'number') {
    // eslint-disable-next-line no-new-wrappers
    return new Number(value);
  }

  if (ty === 'string') {
    // eslint-disable-next-line no-new-wrappers
    return new String(value);
  }
}

function RequireObjectCoercible(value) {
  const ty = typeof value;

  if (ty === 'undefined' || value === null) {
    throw new Error('Cannot coerce value to object');
  }

  return value;
}

export {
  ToPrimitive,
  ToNumber,
  ToLength,
  ToIntegerOrInfinity,
  ToBoolean,
  ToString,
  LengthOfArrayLike,
  ToObject,
  RequireObjectCoercible,
};
