function lengthOfLastWord(s: string): number {
  let str = s.trim().split(" ");
  return str[str.length - 1].length;
}

lengthOfLastWord("   fly me   to   the moon  ");

function lengthOfLastWord_try2(s: string): number {
  const whitespaceChars = [" ", "\t", "\n"];

  const words = s.split(new RegExp(`[${whitespaceChars.join("")}]`));

  let lastWord = "";
  for (let i = words.length - 1; i >= 0; i--) {
    if (words[i].length > 0) {
      lastWord = words[i];
      break;
    }
  }

  return lastWord.length;
}

lengthOfLastWord_try2("   fly me   to   the moon  ");
