export default function main(x: number) {
  try {
    check(x);
  } catch {
    return 10;
  }

  return 0;
}

function check(x: number) {
  if (x > 10) {
    throw new Error('boom');
  }
}
