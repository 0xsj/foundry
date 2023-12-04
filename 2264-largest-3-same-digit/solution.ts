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

function largestGoodInteger(num: string): string {
  const dict = new Set(["999", "888", "777", "666", "555", "444", "333", "222", "111", "000"]);

  for (const match of dict) {
    if (num.includes(match)) {
      return match;
    }
  }

  return "";
}

function largestGoodInteger2(num: string): string {
  let greatest = "";

  for (let i = 0; i < num.length - 2; i++) {
    const current = `${num[i]}${num[i + 1]}${num[i + 2]}`;

    if (current.length === 3 && (Number(current) > Number(greatest) || greatest === "")) {
      greatest = current;
    }
  }

  return greatest;
}

// Example usage:
console.log(largestGoodInteger("6777133339")); // Output: "733"
console.log(largestGoodInteger("6000133339")); // Output: "333"
