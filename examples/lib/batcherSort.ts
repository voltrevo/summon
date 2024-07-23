// https://en.wikipedia.org/wiki/Batcher_odd%E2%80%93even_mergesort
//
// O(n * (log(n))**2)
//
// from 0 to (n-1)
// for p = 1, 2, 4, 8, ... # as long as p < n
//   for k = p, p/2, p/4, p/8, ... # as long as k >= 1
//     for j = mod(k,p) to (n-1-k) with a step size of 2k
//       for i = 0 to min(k-1, n-j-k-1) with a step size of 1
//         if floor((i+j) / (p*2)) == floor((i+j+k) / (p*2))
//           compare and sort elements (i+j) and (i+j+k)

export default function batcherSort(values: number[]) {
  for (let p = 1; p < values.length; p *= 2) {
    for (let k = p; k >= 1; k /= 2) {
      for (let j = k % p; j < values.length - k; j += 2 * k) {
        const iLimit = Math.min(k, values.length - j - k);
        for (let i = 0; i < iLimit; i++) {
          if (Math.floor((i + j) / (p * 2)) === Math.floor((i + j + k) / (p * 2))) {
            const left = values[i + j];
            const right = values[i + j + k];

            if (left > right) {
              values[i + j] = right;
              values[i + j + k] = left;
            }
          }
        }
      }
    }
  }

  return values;
}
