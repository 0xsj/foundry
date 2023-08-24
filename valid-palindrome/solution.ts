const panama = "A man, a plan, a canal: Panama";

function isPalindrome(s: string): boolean {
  // get rid of all special characters if any.
  // we do not care about upper or lower case.
  // split the string into an array of characters, so that we can compute
  // the \s, gets rid of white spaces.

  const stringsArray = s
    .replace(/[&\/\\#,+()$~%.'":*?<>{}\s]/g, "")
    .toLowerCase()
    .split("");
  // once these steps are done, we can take the right side of the array, and left.

  let left = 0;
  let right = stringsArray.length - 1;

  while (left < right) {
    // loop through while the values are equal.other wise we break the loop and return false.

    if (stringsArray[left] !== stringsArray[right]) {
      return false;
    }

    left++;
    right--;
  }
  return true;
}

function isPalindrome_outside(s: string): boolean {
  let strings = s.replace(/[^a-zA-Z0-9]/g, "").toLowerCase();
  let left = 0;
  let right = s.length - 1;

  while (left < right) {
    if (strings[left] !== strings[right]) {
      return false;
    }
    left++;
    right--;
  }
  return true;
}

console.log(isPalindrome_outside(panama));

function isPalindrome(s: string): boolean {
  // Convert the string to lowercase
  s = s.toLowerCase();

  // Create an array of alphanumeric characters
  const alphanumericChars = s.split("").filter(isAlphanumeric);

  return alphanumericChars.reduce((acc, char, index, arr) => {
    const oppositeIndex = arr.length - 1 - index;
    return acc && char === arr[oppositeIndex];
  }, true);
}

function isAlphanumeric(char: string): boolean {
  // Helper function to check if a character is alphanumeric
  return /^[a-z0-9]$/.test(char);
}
