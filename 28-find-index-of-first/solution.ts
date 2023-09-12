/**
 * try 1
 * 1. using javascript includes to see if needle exists in haystack
 * 2. if it does, return the indexof needle
 */
function strStr_try1(haystack: string, needle: string): number {
  if (haystack.includes(needle)) {
    return haystack.indexOf(needle);
  }
  return -1;
}

console.log(strStr_try1("hello", "ll"));
