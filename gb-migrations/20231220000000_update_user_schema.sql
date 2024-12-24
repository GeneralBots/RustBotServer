-- Add password_hash column to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255) NOT NULL DEFAULT '';

-- Update column names if needed
ALTER TABLE users RENAME COLUMN password TO password_hash;

-- Add metadata column to instances table
ALTER TABLE instances 
ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';