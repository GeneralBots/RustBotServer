-- Add role column to users table with a default value
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS role VARCHAR(50) NOT NULL DEFAULT 'user';

-- Create type if you want to use enum
-- DO $$ 
-- BEGIN 
--     IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_role') THEN
--         CREATE TYPE user_role AS ENUM ('admin', 'user', 'guest');
--     END IF;
-- END $$;

-- If you want to use enum instead of varchar, uncomment this:
-- ALTER TABLE users 
-- ALTER COLUMN role TYPE user_role USING role::user_role;