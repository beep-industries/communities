-- Down migration: drop member_roles table, function, and trigger

-- Drop trigger
DROP TRIGGER IF EXISTS member_role_same_server_check ON member_roles;

-- Drop function
DROP FUNCTION IF EXISTS check_member_role_same_server();

-- Drop table
DROP TABLE IF EXISTS member_roles;
