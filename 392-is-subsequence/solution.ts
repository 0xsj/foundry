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

/**
 *
 */
