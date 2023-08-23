const word1 = "anagram";
const word2 = "nagaram";
const word3 = "cat";
const word4 = "tab";

const pair = [word1, word2];
const pair2 = [word3, word4];

class ValidAnagram {
  s: string;
  t: string;
  args?: string[];

  constructor(s: string, t: string, args?: string[]) {
    this.s = s;
    this.t = t;
    this.args = args;
  }

  /**
   * 1. we just compare two strings that we run maniulates through.
   * 2. we split the string, sort it, which would give us a specific sequence of characters
   * 3. we then join the characters array that is sorted to complete / form a new string.
   * 4. we do the same for the second argument / string
   * 5. if the two strings match, it means that it is an anagram
   */

  bruceForce(): boolean {
    return this.s.split("").sort().join("") === this.t.split("").sort().join("");
  }
  hashMap(): boolean {
    if (this.s.length !== this.t.length) {
      // immediate check
      return false;
    }
    const charCount = new Map<string, number>();
    for (const char of this.s) {
      charCount.set(char, (charCount.get(char) || 0) + 1);
    }
    return false;
  }
}

const example = new ValidAnagram("hello", "oellh");
console.log(example.bruceForce());
