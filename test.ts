// // find the mode

// function findMode(nums: number[]): number {
//   let result = 0;

//   const map = new Map<number, number>();

//   console.log(typeof map);

//   console.log(map);

//   for (const num of nums) {
//     if (map.has(num)) {
//       map.set(num, map.get(num)! + 1);
//     } else {
//       map.set(num, 1);
//     }
//   }

//   let frequency = 0;

//   for (const [key, value] of map) {
//     if (frequency < value) {
//       frequency = value;
//       result = key;
//     }
//   }

//   console.log(result);

//   return result;
// }

// // second pass

// // // two pointer
// // function findMode2(arr: number[]): number[] {
// //   let result: number[] = [];
// //   let map = {};

// //   for (let i = 0; i < arr.length; i++) {
// //     if (!map[arr[i]]) {
// //       map[arr[i]] = 0;
// //       map[arr[i]]++;
// //     }
// //   }

// //   return [];
// // }

// findMode([
//   1, 2, 2, 2, 3, 3, 4, 5, 5, 5, 5, 5, 2, 2, 13, 2, 54, 6, 4, 3, 1, 34, 4, 2, 5, 5, 6, 6, 13, 54,
// ]); // 2

// given an array of numbers, return the first reoccuring number

// function reoccuring(nums: number[]): number {
//   const set = new Set();
//   const newSet = new Set(nums);
//   console.log(newSet.keys());

//   for (const num of nums) {
//     if (set.has(num)) {
//       return num;
//     }
//     set.add(num);
//   }

//   return 0;
// }

// reoccuring([1, 2, 3, 4, 5, 1, 2, 4]);
// console.log(reoccuring([1, 2, 3, 4, 5, 2, 1, 4]));
// console.log(reoccuring([1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9]));

// // funciton reoccuring2(nums: number)

// function reoccuring2(arr: number[]): number {
//   let temp = [];

//   for (let i = 0; i < arr.length; i++) {
//     if (temp.indexOf(arr[i]) > 0) {
//       return arr[i];
//     }
//     temp.push(arr[i]);
//   }
// }

// const case1 = "Hello my name is Mongster!"
// const case2 = "Hello my name is Andrew."
// const case3 = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."

// regular
function countVowels(str: string): number {
  let count = 0;

  const lowercase = str.toLowerCase();

  const vowels = ["a", "e", "i", "o", "u"];

  for (let i = 0; i < lowercase.length; i++) {
    if (vowels.includes(lowercase[i])) {
      count++;
    }
  }

  return count;
}

console.log(countVowels("Hello my name is Mongster!"));

// second pass with regex
function countVowels2(str: string): number {
  //   const lowercase = str.toLowerCase();
  const regex = /[aeiou]/gi;

  const matches = str.match(regex);

  return matches ? matches.length : 0;
}

console.log(countVowels2("Hello my name is Mongster!"));

// third pass with set
function countVowels3(str: string): number {
  let count = 0;
  const lower = str.toLowerCase();
  const key = new Set(["a", "e", "i", "o", "u"]);

  for (let i = 0; i < lower.length; i++) {
    if (key.has(lower[i])) {
      count++;
    }
  }

  return count;
}

console.log(
  countVowels3(
    "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  )
);

// fourth pass
// function countVowels4(str: string): number {}
function add(a: number, b: number): number {
  if (b == 0) {
    return a;
  } else {
    return add(a ^ b, (a & b) << 1);
  }
}
