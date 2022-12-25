import unittest


def add(x, y):
    return x + y


def subtract(x, y):
    return x - y


class TestCalculator(unittest.TestCase):
    def test_add(self):
        result = add(10, 5)
        self.assertEqual(result, 15)

    def test_subtract(self):
        result = subtract(10, 5)
        self.assertEqual(result, 5)


if __name__ == '__main__':
    unittest.main()
