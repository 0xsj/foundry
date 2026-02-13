module Inventory where

import Data.Maybe (fromJust)

-- | A product in the inventory.
type ProductID = String
type Price = Double
type Quantity = Int

data Product = Product
  { prodID   :: ProductID
  , prodName :: String
  , prodPrice :: Price
  , prodQty  :: Quantity
  } deriving (Show)

-- | The full inventory is a list of products.
type Inventory = [Product]

-- | Create an empty inventory.
emptyInventory :: Inventory
emptyInventory = []

-- | Add a new product to the inventory.
addProduct :: Product -> Inventory -> Inventory
addProduct p inv = inv ++ [p]

-- | Find a product by ID.
findProduct :: ProductID -> Inventory -> Maybe Product
findProduct pid inv = case filter (\p -> prodID p == pid) inv of
  []    -> Nothing
  (x:_) -> Just x

-- | Get the price of a product by ID.
-- Assumes the product exists.
getPrice :: ProductID -> Inventory -> Price
getPrice pid inv = prodPrice (fromJust (findProduct pid inv))

-- | Update the quantity for a product.
updateQty :: ProductID -> Quantity -> Inventory -> Inventory
updateQty pid newQty inv =
  map (\p -> if prodID p == pid then p { prodQty = newQty } else p) inv

-- | Total value of all inventory (sum of price * quantity).
totalValue :: Inventory -> Double
totalValue inv = sum (map (\p -> prodPrice p * fromIntegral (prodQty p)) inv)

-- | The most expensive product.
mostExpensive :: Inventory -> Product
mostExpensive inv = foldr1 (\a b -> if prodPrice a >= prodPrice b then a else b) inv

-- | Average price across all products.
averagePrice :: Inventory -> Double
averagePrice inv =
  let prices = map prodPrice inv
      total = sum prices
      count = length prices
  in total / fromIntegral count

-- | Products that are low in stock (below threshold).
lowStock :: Quantity -> Inventory -> Inventory
lowStock threshold inv = filter (\p -> prodQty p < threshold) inv

-- | Format a product for display.
showProduct :: Product -> String
showProduct p =
  prodID p ++ " | " ++ prodName p
  ++ " | $" ++ show (prodPrice p)
  ++ " | qty: " ++ show (prodQty p)

-- | Format inventory report.
inventoryReport :: Inventory -> String
inventoryReport inv =
  unlines (map showProduct inv)
  ++ "---\n"
  ++ "Total value: $" ++ show (totalValue inv) ++ "\n"
  ++ "Average price: $" ++ show (averagePrice inv) ++ "\n"
  ++ "Most expensive: " ++ prodName (mostExpensive inv) ++ "\n"
  ++ "Items: " ++ show (length inv)
