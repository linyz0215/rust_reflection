-- -- Add migration script here
-- -- workspace for users
-- CREATE TABLE IF NOT EXISTS workspaces(
--   id bigserial PRIMARY KEY,
--   name varchar(32) NOT NULL UNIQUE,
--   owner_id bigint NOT NULL REFERENCES users(id),
--   created_at timestamptz DEFAULT CURRENT_TIMESTAMP
-- );

-- -- alter users table to add ws_id
-- ALTER TABLE users
--   ADD COLUMN ws_id bigint REFERENCES workspaces(id);

-- -- alter chats table to add ws_id
-- ALTER TABLE chats
--   ADD COLUMN ws_id bigint REFERENCES workspaces(id);

-- -- add super user 0 and workspace 0
-- BEGIN;
-- INSERT INTO users(id, fullname, email, password_hash)
--   VALUES (0, 'super user', 'super@none.org', '');
-- INSERT INTO workspaces(id, name, owner_id)
--   VALUES (0, 'none', 0);
-- UPDATE
--   users
-- SET
--   ws_id = 0
-- WHERE
--   id = 0;
-- COMMIT;

-- -- alter user table to make ws_id not null
-- ALTER TABLE users
--   ALTER COLUMN ws_id SET NOT NULL;

-- This is the file to create COMPANY table and to populate it with 7 records.
-- Just copy and past them on psql prompt.
CREATE TABLE IF NOT EXISTS COMPANY (
   ID INT PRIMARY KEY     NOT NULL,
   NAME           TEXT    NOT NULL,
   AGE            INT     NOT NULL,
   ADDRESS        CHAR(50),
   SALARY         REAL
);
INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (1, 'Paul', 32, 'California', 20000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (2, 'Allen', 25, 'Texas', 15000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (3, 'Teddy', 23, 'Norway', 20000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (4, 'Mark', 25, 'Rich-Mond ', 65000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (5, 'David', 27, 'Texas', 85000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (6, 'Kim', 22, 'South-Hall', 45000.00 );

INSERT INTO COMPANY VALUES (7, 'James', 24, 'Houston', 10000.00 );