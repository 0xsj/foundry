function reverseWords_try1(s: string): string {
  return s.replace(/\s+/g, " ").trim().split(" ").reverse().join(" ");
}

reverseWords_try1("   fly me   to   the moon  ");
