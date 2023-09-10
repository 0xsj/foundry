function isSubsequence_superSlow(s: string, t: string): boolean {
  if (s === "") {
    return true;
  }

  const key = s.split("");
  const bucket: string[] = [];

  for (const char of t) {
    if (char === key[0]) {
      bucket.push(char);
      key.shift();
    }

    if (key.length === 0) {
      return true;
    }
  }

  return false;
}

function isSubsequence_2(s: string, t: string): boolean {
  if (s == "") {
    return true;
  }

  const key = s.split("");
  let keyIndex = 0;

  for (const char of t) {
    if (char === key[keyIndex]) {
      keyIndex++;
      if (keyIndex === key.length) {
        return true;
      }
    }
  }
  return false;
}

/**
 *
 */

function isSubsequence_hashMap(s: string, t: string): boolean {
  const charMap = new Map();

  for (let i = 0; i < t.length; i++) {
    if (!charMap.has(t[i])) {
      charMap.set(t[i], []);
    }
    charMap.get(t[i]).push(i);
  }

  let prevIndex = -1;

  for (const char of s) {
    if (!charMap.has(char)) {
      return false;
    }
    const charIndex = charMap.get(char);
    let found = false;

    for (const index of charIndex) {
      if (index > prevIndex) {
        prevIndex = index;
        found = true;
        break;
      }
    }
    if (!found) {
      return false;
    }
  }
  return true;
}

/**
 *
 */

// function isSubsequence_map(s: string, t: string): boolean {
//   const tIndices = [...t].map((char, index) => ({ char, index }));
//   let prevIndex = -1;

//   return [...s].every((char) => {
//     const charIndex = tIndices.find(({ char, index }) => char === char && index > prevIndex);
//     if (!charIndex) {
//       return false;
//     }
//     prevIndex = charIndex.index;
//     return true;
//   });
// }
// /**
//  *
//  */

// function isSubsequence_set(s: string, t: string): boolean {
//   const sSet = new Set([...s]);

//   for (const char of t) {
//     if (sSet.has(char)) {
//       sSet.delete(char);
//       if (sSet.size === 0) {
//         return true;
//       }
//     }
//   }

//   return sSet.size === 0;
// }
/**
 *
 */

function isSubsequence_twoPointer(s: string, t: string): boolean {
  let sPointer = 0;
  let tPointer = 0;

  while (sPointer < s.length && tPointer < t.length) {
    if (s[sPointer] === t[tPointer]) {
      sPointer++;
    }
    tPointer++;
  }

  return sPointer === s.length;
}
/**
 * divide and conquer
 * 1.
 */

function isSubsequence(s: string, t: string): boolean {
  if (s.length === 0) {
    return true;
  }
  if (t.length === 0) {
    return false;
  }
  if (s[0] === t[0]) {
    return isSubsequence(s.slice(1), t.slice(1));
  } else {
    return isSubsequence(s, t.slice(1));
  }
}
