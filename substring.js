// // Given an array output the closest subset array
// const case1 = ["hello", "world", "world", "at", "this", "is", "of", "not", "think", "okay", "got"];
// const keywords = new Set(["world", "this", "not"]);
// // output [2, 3, 4, 5, 6, 7]

// function findClosest(arr, keywords) {
//   let indices = [];

//   for (let i = 0; i < arr.length; i++) {
//     if (keywords.has(arr[i])) {
//       indices.push(i);
//     }
//   }

//   // base case
//   if (indices.length === 0) {
//     return [];
//   }

//   let start = 0;
//   let end = 0;
//   let closestSet = [];
//   let minDistance = Infinity;

//   while (end < arr.length) {
//     // subset
//     let subset = arr.slice(start, end + 1);
//     let distance = subset.reduce((acc, val) => {
//       if (keywords.has(val)) {
//         return acc;
//       }
//       return acc + 1;
//     }, 0);

//     if (distance < minDistance) {
//       minDistance = distance;
//       closestSet = subset;
//     }

//     if (distance === 0) {
//       start++;
//     } else {
//       end++;
//     }
//   }

//   return closestSet.map((_, i) => indices[0] + i);
// }

// console.log(findClosest(case1, keywords));

// // try 2

// function findClosest2(arr, keywords) {
//   let indices = [];

//   for (let i = 0; i < arr.length; i++) {
//     if (keywords.has(arr[i])) {
//       indices.push(i);
//     }
//   }

//   if (indices.length === 0) {
//     return [];
//   }

//   let minDistance = Infinity;
//   let closestSubset = [];

//   for (let i = 0; i < arr.length - indices.length + 1; i++) {
//     let subset = arr.slice(i, i + indices.length);
//     let distance = subset.reduce((acc, val) => {
//       if (keywords.has(val)) {
//         return acc;
//       }
//       return acc + 1;
//     }, 0);

//     if (distance < minDistance) {
//       minDistance = distance;
//       closestSubset = subset;
//     }
//   }

//   return closestSubset.map((_, i) => indices[0] + i);
// }

// console.log(findClosest2(case1, keywords));
