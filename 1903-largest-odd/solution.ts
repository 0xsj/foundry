// try 1, doesn't work.

// function largestOddNumber(num: string): string {
//   const set = new Set(["1", "3", "5", "7", "9"]);

//   // base case
//   const number = Number(num);
//   if (number % 2 === 1) {
//     return num;
//   } else {
//     for (const match of set) {
//       if (num.includes(match)) {
//         console.log("match:    ", match);
//         return match;
//       }
//     }
//   }

//   return "";
// }

function largestOddNumber(num: string): string {
  const number = Number(num);
  const length = num.length;
  let result = "";
  if (number % 2 == 1) {
    return num;
  }

  for (let i = length - 1; i >= 0; i--) {
    const digit = Number(i);
    if (digit % 2 == 1) {
      return num.substring(0, i + 1);
    }
  }

  return result;
}
// boyer moore doesn't work.
function largestOddNumber2(num: string): string {
  const pattern = /[13579]/;
  let i = num.length - 1;

  while (i < num.length) {
    if (pattern.test(num[i])) {
      return num.substring(0, i + 1);
    }
    const jump = Math.max(1, i - num.lastIndexOf(pattern.exec(num.substring(0, i))?.[0] || ""));
    i += jump;
  }

  return "";
}

//two pointer;
function largestOddNumber3(num: string): string {
  let start = num.length - 1;
  let end = start;

  while (start >= 0) {
    const digit = Number(num[start]);

    if (digit % 2 === 1) {
      return num.substring(start, end + 1);
    }

    start--;
  }

  return "";
}

largestOddNumber3("52");
largestOddNumber3("4206");
largestOddNumber3("35427");
