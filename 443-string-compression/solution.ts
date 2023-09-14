// function compress_map(chars: string[]): number {
//   const set = new Set();
//   const map = new Map();

//   for (const char of chars) {
//     map.set(char, (map.get(char) || 0) + 1);
//   }

//   let result: string[] = [];

//   for (const [char, count] of map.entries()) {
//     result.push(char);
//     if (count > 1) {
//       result.push(count.toString());
//     }
//   }

//   return result.length;
// }

// compress(["a", "a", "b", "b", "c", "c", "c"]);

// function compress(chars: string[]): number {
//   let result = [];
//   let current = chars[0]
// }

const compress_map = (chars: string[]): number => {
  let writeIndex = 0;
  let current = chars[0];
  let charCount = 1;

  for (let i = 1; i < chars.length; i++) {
    if (chars[i] === current) {
      charCount++;
    } else {
      chars[writeIndex++] = current;
      if (charCount > 1) {
        const countString = charCount.toString();
        for (let j = 0; j < countString.length; j++) {
          chars[writeIndex++] = countString[j];
        }
      }
      current = chars[i];
      charCount = 1;
    }
  }

  chars[writeIndex++] = current;
  if (charCount > 1) {
    const countString = charCount.toString();
    for (let j = 0; j < countString.length; j++) {
      chars[writeIndex++] = countString[j];
    }
  }

  chars.length = writeIndex;

  return writeIndex;
};

console.log(compress_map(["a", "a", "b", "b", "c", "c", "c"]));
