//! test [4, 32, 8, 10, 50, 5, 20, 25, 38, 58, 93] => [25]

import median from "./lib/median.ts";

export default function median11(
  x0: number,
  x1: number,
  x2: number,
  x3: number,
  x4: number,
  x5: number,
  x6: number,
  x7: number,
  x8: number,
  x9: number,
  x10: number,
) {
  return median([
    x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
  ]);
}