// /**
//  * try 1
//  * 1. using javascript includes to see if needle exists in haystack
//  * 2. if it does, return the indexof needle
//  */
// function strStr_try1(haystack: string, needle: string): number {
//   if (haystack.includes(needle)) {
//     return haystack.indexOf(needle);
//   }
//   return -1;
// }

// /**
//  *
//  */

// function strStr_try2(haystack: string, needle: string): number {
//   for (let i = 0; i <= haystack.length - needle.length; i++) {
//     if (haystack.substring(i, i + needle.length)) {
//       return i;
//     }
//   }

//   return -1;
// }

// /**
//  * KMP
//  */

// function strStr(haystack: string, needle: string): number {
//   if (needle === "") return 0;

//   // Compute longest suffix-prefix table for needle
//   var lsp = [0]; // Base case
//   for (var i = 1, j = 0; i < needle.length; ) {
//     if (needle[i] === needle[j]) {
//       j++;
//       lsp[i] = j;
//       i++;
//     } else {
//       if (j !== 0) {
//         j = lsp[j - 1];
//       } else {
//         lsp[i] = 0;
//         i++;
//       }
//     }
//   }

//   // Walk through haystack and perform KMP search
//   for (var i = 0, j = 0; i < haystack.length; ) {
//     if (haystack[i] === needle[j]) {
//       i++;
//       j++;
//       if (j === needle.length) {
//         return i - j;
//       }
//     } else {
//       if (j !== 0) {
//         j = lsp[j - 1];
//       } else {
//         i++;
//       }
//     }
//   }

//   return -1;
// }

// /**
//  * Boyer Moore
//  */

// function strStr_boyer(haystack: string, needle: string): number {
//   const m = needle.length;
//   const n = haystack.length;

//   function computeGoodSuffixShift(j, m, suffix, prefix) {
//     const lengthOfSuffix = m - 1 - j;
//     if (suffix[lengthOfSuffix] > 0) {
//       return m - suffix[lengthOfSuffix] - lengthOfSuffix;
//     }
//     for (let r = j + 2; r < m; r++) {
//       if (prefix[m - r]) {
//         return r;
//       }
//     }
//     return m;
//   }

//   if (m === 0) return 0;

//   // Precompute the bad character table
//   const badChar = new Array(256).fill(-1);
//   for (let i = 0; i < m; i++) {
//     badChar[needle.charCodeAt(i)] = i;
//   }

//   // Precompute the good suffix table
//   const suffix = new Array(m).fill(0);
//   const prefix = new Array(m).fill(false);
//   for (let i = m - 1; i >= 0; i--) {
//     if (i < m - 1) {
//       const j = m - 1 - suffix[m - 1 - i];
//       if (j >= 0) {
//         prefix[j] = true;
//       }
//     }
//     while (i - suffix[i] >= 0 && needle[i - suffix[i]] === needle[i]) {
//       suffix[i]++;
//     }
//   }

//   // Searching
//   let s = 0;
//   while (s <= n - m) {
//     let j = m - 1;
//     while (j >= 0 && needle[j] === haystack[s + j]) {
//       j--;
//     }
//     if (j < 0) {
//       // Match found at position s
//       return s;
//     } else {
//       // Shift based on bad character heuristic and good suffix table
//       const badCharShift = j - badChar[haystack.charCodeAt(s + j)];
//       const goodSuffixShift = computeGoodSuffixShift(j, m, suffix, prefix);
//       s += Math.max(badCharShift, goodSuffixShift);
//     }
//   }
//   return -1; // needle not found
// }

// /**
//  * z algorithm
//  */

// function zAlgorithm(haystack: string, needle: string): number {
//   const concat = needle + "$" + haystack;
//   const n = concat.length;
//   const needleLength = needle.length;
//   const z = new Array(n).fill(0);

//   let l = 0;
//   let r = 0;
//   for (let i = 1; i < n; i++) {
//     if (i > r) {
//       l = r = i;
//       while (r < n && concat[r - l] === concat[r]) {
//         r++;
//       }
//       z[i] = r - l;
//       r--;
//     } else {
//       const k = i - l;
//       if (z[k] < r - i + 1) {
//         z[i] = z[k];
//       } else {
//         l = i;
//         while (r < n && concat[r - l] === concat[r]) {
//           r++;
//         }
//         z[i] = r - l;
//         r--;
//       }
//     }
//     if (z[i] === needleLength) {
//       return i - needleLength - 1; // Match found at position i - needleLength - 1
//     }
//   }
//   return -1; // needle not found
// }

/**
 * Robin Carp
 */

function rabinKarp(haystack: string, needle: string): number {
  const n = haystack.length;
  const m = needle.length;

  function hash(str: string) {
    let hashValue = 0;
    for (let i = 0; i < str.length; i++) {
      hashValue += str.charCodeAt(i);
    }
    return hashValue;
  }

  function rehash(oldHash: number, str: string, oldIndex: number, newIndex: number) {
    console.log(typeof oldHash, str, oldIndex, newIndex);
    return oldHash - str.charCodeAt(oldIndex) + str.charCodeAt(newIndex);
  }

  if (m === 0) return 0;

  const hashneedle = hash(needle);
  let hashhaystack = hash(haystack.substring(0, m));

  for (let i = 0; i <= n - m; i++) {
    if (hashneedle === hashhaystack && haystack.substring(i, i + m) === needle) {
      return i;
    }
    if (i < n - m) {
      hashhaystack = rehash(hashhaystack, haystack, i, i + m);
    }
  }

  return -1; // needle not found
}

console.log(rabinKarp("hello", "ll"));
