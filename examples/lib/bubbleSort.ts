export default function bubbleSort(values: number[]) {
  let left = 0;
  let right = values.length - 1;

  while (true) {
    if (right <= left) {
      break;
    }

    for (let i = left; i < right; i++) {
      if (values[i] > values[i + 1]) {
        const tmp = values[i];
        values[i] = values[i + 1];
        values[i + 1] = tmp;
      }
    }

    right--;

    if (right <= left) {
      break;
    }

    for (let i = right - 1; i >= left; i--) {
      if (values[i] > values[i + 1]) {
        const tmp = values[i];
        values[i] = values[i + 1];
        values[i + 1] = tmp;
      }
    }

    left++;
  }

  return values;
}
