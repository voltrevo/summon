import batcherSort from "./batcherSort.ts";

export default function median(input: number[]) {
  const sorted = batcherSort(input); // faster than bubbleSort(input)

  const mid = (input.length - 1) / 2;

  if (mid === Math.floor(mid)) {
    return sorted[mid];
  }

  return (sorted[Math.floor(mid)] + sorted[Math.ceil(mid)]) / 2;
}
