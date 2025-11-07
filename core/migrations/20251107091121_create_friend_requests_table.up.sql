-- Create the friend_requests table
CREATE TABLE friend_requests (
    user_id_requested UUID NOT NULL,
    user_id_invited UUID NOT NULL,
    status SMALLINT DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id_requested, user_id_invited)
);