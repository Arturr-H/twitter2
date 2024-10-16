-- Step 1: Drop the foreign key constraint for user_id
ALTER TABLE likes
DROP CONSTRAINT likes_user_id_fkey;

-- Step 2: Alter the column type from INT to BIGINT for user_id
ALTER TABLE likes
ALTER COLUMN user_id TYPE BIGINT;

-- Step 3: Add the foreign key constraint back for user_id
ALTER TABLE likes
ADD CONSTRAINT likes_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;
