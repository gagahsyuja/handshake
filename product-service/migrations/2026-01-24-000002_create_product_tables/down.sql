-- Drop indexes
DROP INDEX IF EXISTS idx_products_created_at;
DROP INDEX IF EXISTS idx_products_status;
DROP INDEX IF EXISTS idx_products_category;
DROP INDEX IF EXISTS idx_products_seller;

-- Drop tables
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS categories;
