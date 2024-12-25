-- Add password_hash column to users table if it doesn't exist
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255) NOT NULL DEFAULT '';

-- Rename existing password column to password_hash if it exists
DO $$ 
BEGIN 
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'users' AND column_name = 'password') THEN
        ALTER TABLE users RENAME COLUMN password TO password_hash;
    END IF;
END $$;

-- Add metadata column to instances table
ALTER TABLE instances 
ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';
