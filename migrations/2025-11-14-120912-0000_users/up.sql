-- Your SQL goes here
-- Create the UserRole ENUM type


-- Create the Users table
CREATE TABLE users (
    -- Unique Identifier (Primary Key)
    id UUID PRIMARY KEY NOT NULL,

    -- Authentication/Identity fields
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT,

    -- Metadata/Display fields
    display_name VARCHAR(100) NOT NULL,
    
    -- Role and Status
    user_role VARCHAR(100) NOT NULL DEFAULT 'Standard',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Timestamps
    last_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create an index on the username and email for faster lookups
CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_email ON users (email);