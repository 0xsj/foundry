-- TRY 1
SELECT product_id FROM Products WHERE low_fats = 'Y' AND recyclable = 'Y';


-- TRY 2

SELECT p.product_id FROM Products as p INNER JOIN Products AS q ON p.product_id = q.product_id AND p.low_fats = 'Y' AND q.recycleable = 'Y'