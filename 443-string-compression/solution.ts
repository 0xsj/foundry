function compress_map(chars: string[]): number {
  const set = new Set();
  const map = new Map();

  for (const char of chars) {
    map.set(char, (map.get(char) || 0) + 1);
  }

  let result: string[] = [];

  for (const [char, count] of map.entries()) {
    result.push(char, count.toString());
  }

  console.log(result);

  return result.length;
}

compress(["a", "a", "b", "b", "c", "c", "c"]);

function compress(chars: string[]): number {
  let result = [];
  let current = chars[0];
  let count = 1;

  // for (let i = 1; i < chars.length; i++)  {
  //   if (chars[i])
  // }

  return result.length;
}

console.log(compress_map(["a", "a", "b", "b", "c", "c", "c"]));
