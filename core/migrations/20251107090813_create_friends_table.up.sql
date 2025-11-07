-- Create the friends table
CREATE TABLE friends (
    user_id_1 UUID NOT NULL,
    user_id_2 UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id_1, user_id_2)
);