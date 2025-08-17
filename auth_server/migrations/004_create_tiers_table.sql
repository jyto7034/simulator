-- Tiers master data table
CREATE TABLE IF NOT EXISTS tiers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    order_key INT UNIQUE NOT NULL,
    icon_url TEXT
);
COMMENT ON TABLE tiers IS 'Game tier/rank definition master table';
-- Insert sample tier data
INSERT INTO tiers (id, name, order_key, icon_url) VALUES
(1, 'Bronze', 1, '/icons/tiers/bronze.png'),
(2, 'Silver', 2, '/icons/tiers/silver.png'),
(3, 'Gold', 3, '/icons/tiers/gold.png'),
(4, 'Platinum', 4, '/icons/tiers/platinum.png'),
(5, 'Diamond', 5, '/icons/tiers/diamond.png')
ON CONFLICT (id) DO NOTHING;