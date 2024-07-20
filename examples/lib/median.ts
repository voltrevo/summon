import bubbleSort from "./bubbleSort.ts";

export default function median(input: number[]) {
  const sorted = bubbleSort(input);

  const mid = (input.length - 1) / 2;

  if (mid === Math.floor(mid)) {
    return sorted[mid];
  }

  return (sorted[Math.floor(mid)] + sorted[Math.ceil(mid)]) / 2;
}
