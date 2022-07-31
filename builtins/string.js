/* eslint-disable no-unused-vars */
/* eslint-disable no-constructor-return */
import { RequireObjectCoercible, ToIntegerOrInfinity, ToString } from './abstract';

class String {
  constructor(value) {
    return ToString(value);
  }

  at(index) {
    const O = RequireObjectCoercible(this);
    const S = ToString(O);
    const len = S.length;
    const relativeIndex = ToIntegerOrInfinity(index);
    const k = relativeIndex >= 0 ? relativeIndex : len + relativeIndex;

    return S.substring(k, k + 1);
  }

  charAt(pos) {
    const O = RequireObjectCoercible(this);
    const S = ToString(O);
    const position = ToIntegerOrInfinity(pos);

    const size = S.length;
    if (position < 0 || position >= size) {
      return '';
    }

    return S.substring(position, position + 1);
  }

  // TODO: charCodeAt, codePointAt

  concat() { }
}
