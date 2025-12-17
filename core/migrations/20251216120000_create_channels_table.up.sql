-- Create channel_type enum
CREATE TYPE channel_type AS ENUM ('serverText', 'serverVoice', 'serverFolder', 'private');

-- Create the channels table
CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(30) NOT NULL,
    server_id UUID REFERENCES servers(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES channels(id) ON DELETE SET NULL,
    channel_type channel_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    -- Constraint: server channels must have a server_id, private channels must not
    CONSTRAINT channel_server_constraint CHECK (
        (channel_type = 'private' AND server_id IS NULL) OR
        (channel_type != 'private' AND server_id IS NOT NULL)
    )
);

-- Create trigger to automatically update updated_at on UPDATE
CREATE TRIGGER update_channels_updated_at
    BEFORE UPDATE ON channels
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create indexes for common queries
CREATE INDEX idx_channels_parent_id ON channels(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_channels_server_id ON channels(server_id) WHERE server_id IS NOT NULL;
