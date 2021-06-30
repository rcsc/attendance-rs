CREATE TABLE IF NOT EXISTS users(
   uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
   full_name VARCHAR(200) NOT NULL,
   email VARCHAR(500) NOT NULL,
   phone_number VARCHAR(14)
);

CREATE TABLE IF NOT EXISTS attendance (
   id SERIAL PRIMARY KEY,
   user_uuid UUID REFERENCES users (uuid) NOT NULL,
   in_time TIMESTAMP WITH TIME ZONE NOT NULL,
   out_time TIMESTAMP WITH TIME ZONE
);
