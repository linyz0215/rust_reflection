-- insert users
INSERT INTO users (name, email, password, type) 
VALUES
('linyz', 'linyz@shanghaitech.edu.cn', 'password1', 'admin'),
('uu', 'uu@shanghaitech.edu.cn', 'password2', 'regular'),
('alice', 'alice@shanghaitech.edu.cn', 'password3', 'regular')
RETURNING name, email, password, type;

-- insert chat
INSERT INTO chats (chat_name, type, members)
VALUES 
('General_Chat', 'single', ARRAY[1, 2]),
('Group_Chat', 'group', ARRAY[1,2,3])
RETURNING chat_id, chat_name, type, members;