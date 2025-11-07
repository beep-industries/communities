-- Add down migration script here

-- Drop the trigger first
DROP TRIGGER IF EXISTS update_servers_updated_at ON servers;

-- Drop the table
DROP TABLE IF EXISTS servers;

-- Note: Not dropping the function update_updated_at_column() as it might be used by other tables
