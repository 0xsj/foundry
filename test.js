// let test1 = [1, 3, 9]; // 2
// let test2 = [-10, -20, -5, -55, -60, 100]; //  5
// let test3 = [2, 100, 8, 20]; // 6
// let test4 = [15, 98, 22, 74, 44, 72]; // 2
// //
// function findSmallestInterval(numbers) {
//   let smallestInterval = Math.abs(numbers[1] - numbers[0]);

//   numbers.sort((a, b) => a - b);

//   let left = 0;
//   let right = 1;

//   while (right < numbers.length) {
//     const currentInterval = Math.abs(numbers[right] - numbers[left]);
//     smallestInterval = Math.min(smallestInterval, currentInterval);

//     left++;
//     right++;
//   }

//   return smallestInterval;
// }

// test1 = [2, 100, 8, 20];

// //
// function findSmallestInterval2(numbers) {
//   let smallestInterval = Infinity;

//   for (let i = 1; i < numbers.length; i++) {
//     const currentInterval = Math.abs(numbers[i] - numbers[i - 1]);
//     smallestInterval = Math.min(smallestInterval, currentInterval);
//   }

//   return smallestInterval;
// }

// console.log(findSmallestInterval2(test3));

// function findSmallestInterval(numbers) {
//   let smallestInterval = Infinity;

//   // Iterate through all pairs of numbers to find the smallest interval
//   for (let i = 0; i < numbers.length - 1; i++) {
//     for (let j = i + 1; j < numbers.length; j++) {
//       const currentInterval = Math.abs(numbers[j] - numbers[i]);
//       smallestInterval = Math.min(smallestInterval, currentInterval);
//     }
//   }

//   return smallestInterval;
// }

// // sorted
// function findSmallestInterval(numbers) {
//   numbers.sort((a, b) => a - b);

//   let smallestInterval = Infinity;

//   // Iterate through the sorted array to find the smallest interval
//   for (let i = 1; i < numbers.length; i++) {
//     const currentInterval = Math.abs(numbers[i] - numbers[i - 1]);
//     smallestInterval = Math.min(smallestInterval, currentInterval);
//   }

//   return smallestInterval;
// }

// console.log(findSmallestInterval(test1));
// console.log(findSmallestInterval(test2));
// console.log(findSmallestInterval(test3));
// console.log(findSmallestInterval(test4));

// function findHiggs(particles) {
//   // Ensure there are enough particles for pairs
//   if (particles.length < 2) {
//     console.log("Insufficient particles for analysis.");
//     return [];
//   }

//   // Iterate through each pair of particles
//   for (let i = 0; i < particles.length - 1; i++) {
//     for (let j = i + 1; j < particles.length; j++) {
//       const particle1 = particles[i];
//       const particle2 = particles[j];

//       // Check conditions for potential Higgs boson
//       if (
//         isSimilarMass(particle1.mass, particle2.mass) &&
//         isOppositeCharge(particle1.charge, particle2.charge)
//       ) {
//         return [i, j]; // Return the indices of the two particles
//       }
//     }
//   }

//   return []; // Return an empty array if no Higgs boson pairs are found
// }

// // Example particle structure: { x: 1, y: 2, z: 3, mass: 30, charge: -1 }
// // You can modify the properties based on your actual data

// // Check if masses are similar within a certain threshold
// function isSimilarMass(mass1, mass2) {
//   const massThreshold = 5; // Adjust this threshold as needed
//   return Math.abs(mass1 - mass2) < massThreshold;
// }

// // Check if charges are opposite
// function isOppositeCharge(charge1, charge2) {
//   return charge1 * charge2 === -1;
// }

// let distance = [
//   [3, 2],
//   [-4, -1],
//   [1, -1],
//   [8, 1],
//   [0, 5],
// ]; // [0,2]

// console.logfindHiggs(distance);

function findMode2(arr) {
  let map = {};
  let result = [];

  for (let i = 0; i < arr.length; i++) {
    if (!map[arr[i]]) {
      map[arr[i]] = 0;
    }
    map[arr[i]] += 1;
  }

  let largest = -1;
  let biggestKey = -1;

  Object.keys(map).forEach((key) => {
    let value = map[key];
    if (value > largest) {
      result = [];
      largest = value;
      result.push(key);
    } else if (value === largest) {
      result.push(key);
    }
  });

  console.log(map);

  //   const set = new Set(result);

  //   let really = Array.from(set);
  //   console.log(really);

  console.log(result);

  return result;
}

findMode2([
  1, 2, 2, 2, 3, 3, 4, 5, 5, 5, 5, 5, 2, 2, 13, 2, 54, 6, 4, 3, 1, 34, 4, 2, 5, 5, 6, 6, 13, 54,
  1399,
]); // 2
