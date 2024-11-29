CREATE TABLE users (
    id SERIAL primary key,
    name varchar(100) NOT NULL,
    email varchar(100) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)