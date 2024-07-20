export default function bubbleSort(values: number[]) {
  let left = 0;
  let right = values.length - 1;

  while (true) {
    if (right - left > 1) {
      break;
    }

    for (let i = left; i < right - 1; i++) {
      if (values[i] > values[i + 1]) {
        const tmp = values[i];
        values[i] = values[i + 1];
        values[i + 1] = tmp;
      }
    }

    right--;

    if (right - left > 1) {
      break;
    }

    for (let i = right - 2; i >= left; i--) {
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
