// nope
function canConstruct2(ransomNote: string, magazine: string): boolean {
  const set = new Set(magazine.split(""));

  for (let char of ransomNote) {
    if (set.has(char)) {
      set.delete(char);
    } else {
      return false;
    }
  }

  return true;
}

// try 2
function canConstruct(ransomNote: string, magazine: string): boolean {
  const charCount = new Map<string, number>();

  for (const char of magazine) {
    charCount.set(char, (charCount.get(char) || 0) + 1);
  }

  for (const char of ransomNote) {
    if (!charCount.has(char) || charCount.get(char) === 0) {
      return false;
    }

    charCount.set(char, charCount.get(char)! - 1);
  }

  return true;
}

console.log(canConstruct("aa", "bb"));
console.log(canConstruct("aa", "aabb"));
console.log(canConstruct("bg", "efjbdfbdgfjhhaiigfhbaejahgfbbgbjagbddfgdiaigdadhcfcj"));
