function lengthOfLastWord(s: string): number {
  let str = s.trim().split(" ");
  return str[str.length - 1].length;
}

lengthOfLastWord("   fly me   to   the moon  ");
