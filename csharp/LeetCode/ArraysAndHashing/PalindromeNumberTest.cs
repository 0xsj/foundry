// csharp/LeetCode/ArraysAndHashing/PalindromeNumberTest.cs

using Xunit;
using System.Diagnostics;

namespace LeetCode.ArraysAndHashing
{
    public class PalindromeNumberTest
    {
        [Fact]
        public void TestSingleDigitNumbers()
        {
            Assert.True(PalindromeNumber.IsPalindrome(0));
            Assert.True(PalindromeNumber.IsPalindrome(7));
            Assert.True(PalindromeNumber.IsPalindrome(9));
        }

        [Fact]
        public void TestNegativeNumbers()
        {
            Assert.False(PalindromeNumber.IsPalindrome(-121));
            Assert.False(PalindromeNumber.IsPalindrome(-1));
            Assert.False(PalindromeNumber.IsPalindrome(-101));
        }

        [Fact]
        public void TestNumbersEndingInZero()
        {
            Assert.False(PalindromeNumber.IsPalindrome(10));
            Assert.False(PalindromeNumber.IsPalindrome(100));
            Assert.False(PalindromeNumber.IsPalindrome(1000));
        }

        [Fact]
        public void TestEvenLengthPalindromes()
        {
            Assert.True(PalindromeNumber.IsPalindrome(11));
            Assert.True(PalindromeNumber.IsPalindrome(1221));
            Assert.True(PalindromeNumber.IsPalindrome(123321));
        }

        [Fact]
        public void TestOddLengthPalindromes()
        {
            Assert.True(PalindromeNumber.IsPalindrome(121));
            Assert.True(PalindromeNumber.IsPalindrome(12321));
            Assert.True(PalindromeNumber.IsPalindrome(1234321));
        }

        [Fact]
        public void TestNonPalindromes()
        {
            Assert.False(PalindromeNumber.IsPalindrome(12));
            Assert.False(PalindromeNumber.IsPalindrome(123));
            Assert.False(PalindromeNumber.IsPalindrome(1234));
        }

        [Fact]
        public void TestLargerNumbers()
        {
            Assert.True(PalindromeNumber.IsPalindrome(9999999));
            Assert.True(PalindromeNumber.IsPalindrome(12344321));
            Assert.False(PalindromeNumber.IsPalindrome(1234567));
            Assert.False(PalindromeNumber.IsPalindrome(9876543));
        }

        [Fact]
        public void TestPerformance1K()
        {
            var sw = Stopwatch.StartNew();
            
            for (int i = 0; i < 1000; i++)
            {
                PalindromeNumber.IsPalindrome(i);
            }
            
            sw.Stop();
            Console.WriteLine($"[1K iterations] Execution time: {sw.Elapsed.TotalMilliseconds:F3}ms");
            
            Assert.True(sw.Elapsed.TotalMilliseconds < 100);
        }

        [Fact]
        public void TestPerformance10K()
        {
            var sw = Stopwatch.StartNew();
            
            for (int i = 0; i < 10000; i++)
            {
                PalindromeNumber.IsPalindrome(i % 1000000);
            }
            
            sw.Stop();
            Console.WriteLine($"[10K iterations] Execution time: {sw.Elapsed.TotalMilliseconds:F3}ms");
            
            Assert.True(sw.Elapsed.TotalMilliseconds < 200);
        }

        [Fact]
        public void TestPerformance100K()
        {
            var sw = Stopwatch.StartNew();
            
            for (int i = 0; i < 100000; i++)
            {
                PalindromeNumber.IsPalindrome(i % 10000000);
            }
            
            sw.Stop();
            Console.WriteLine($"[100K iterations] Execution time: {sw.Elapsed.TotalMilliseconds:F3}ms");
            
            Assert.True(sw.Elapsed.TotalMilliseconds < 1000);
        }
    }
}