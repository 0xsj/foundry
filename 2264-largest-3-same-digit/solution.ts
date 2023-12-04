// // doesn't work
// function largestGoodInteger(num: string): string {
//   const map = new Map();

//   for (const char of num) {
//     if (map.has(char)) {
//       map.set(char, map.get(char) + 1);
//     } else {
//       map.set(char, 1);
//     }
//   }

//   let largestKey = "";
//   for (const [key, value] of map) {
//     if (value >= 3 && key > largestKey) {
//       largestKey = key;
//     }
//   }

//   return largestKey.repeat(3);
// }

// // try 2, doesn't work
// function largestGoodInteger2(num: string): string {
//   let match = "";

//   // create a set or a map, where we will have unique values. the range possible is 0~9
//   const map = new Map<string, number>();

//   for (const char of num) {
//     if (map.has(char)) {
//       map.set(char, map.get(char)! + 1);
//     } else {
//       map.set(char, 1);
//     }
//   }

//   for (const [key, value] of map) {
//     if (value >= 3) {
//       // num.
//     }
//   }

//   // search in the set where the occurances/ values are greater than 3

//   // go through the num string, and search for substrings using .repeat()

//   return match;
// }

console.log(largestGoodInteger("6777133339"));
console.log(largestGoodInteger("6000133339"));
