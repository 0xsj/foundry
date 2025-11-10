namespace LeetCode.ArraysAndHashing
{
    public static class PalindromeNumber
    {
        public static bool IsPalindrome(int x)
        {
            if (x < 0)
            {
                return false;
            }

            if (x % 10 == 0 && x != 0)
            {
                return false;
            }

            string numStr = x.ToString();

            int left = 0;
            int right = numStr.Length - 1;

            while (left < right)
            {
                if (numStr[left] != numStr[right])
                {
                    return false;
                }

                left++;
                right--;
            }
            
            return true;
        }
    }
}