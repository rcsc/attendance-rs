ALTER TABLE users ADD CONSTRAINT attendance_unique_constraint UNIQUE  (full_name, email, phone_number);
