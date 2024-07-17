const iterations = 3;

export default function main(input: number) {
  let res = 0;

  for (let i = 0; i < iterations; i++) {
    res += input;
  }

  return res;
}
