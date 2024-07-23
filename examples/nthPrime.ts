//! test [0] => [0]
//! test [1] => [2]
//! test [2] => [3]
//! test [3] => [5]
//! test [10] => [29]
//! test [11] => [0]

const limit = 10;

export default function main(n: number) {
  let p = 0;
  let i = 0;

  for (; i < n && i < limit; i++) {
    p = nextPrime(p);
  }

  if (i < n) {
    return 0;
  }

  return p;
}

export function nextPrime(x: number) {
  if (x < 2) {
    return 2;
  }

  x += (x % 2) + 1; // next odd number

  while (!isOddPrime(x)) {
    x += 2;
  }

  return x;
}

function isOddPrime(x: number) {
  for (let div = 3; div * div <= x; div += 2) {
    if (x % div === 0) {
      return false;
    }
  }

  return true;
}
