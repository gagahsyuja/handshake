-- Drop indexes
DROP INDEX IF EXISTS idx_orders_created_at;
DROP INDEX IF EXISTS idx_orders_product;
DROP INDEX IF EXISTS idx_orders_seller;
DROP INDEX IF EXISTS idx_orders_buyer;
DROP INDEX IF EXISTS idx_locations_user;

-- Drop tables
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS locations;
