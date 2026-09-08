module MathUtils where

-- Factorial function with pattern matching
factorial :: Integer -> Integer
factorial 0 = 1
factorial n | n > 0 = n * factorial (n - 1)
            | otherwise = error "factorial of negative number"

-- Safe division returning Maybe
safeDiv :: Double -> Double -> Maybe Double
safeDiv _ 0.0 = Nothing
safeDiv x y   = Just (x / y)

-- Check if a number is prime
isPrime :: Integer -> Bool
isPrime n
  | n <= 1 = False
  | otherwise = null [ x | x <- [2 .. floor (sqrt (fromIntegral n))], n `mod` x == 0 ]

-- Fibonacci sequence generator
fib :: Int -> Integer
fib n = fibs !! n
  where fibs = 0 : 1 : zipWith (+) fibs (tail fibs)
