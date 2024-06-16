-- insert 3 workspaces
INSERT INTO workspaces (name, owner_id)
VALUES
('acme', 0),
('foo', 0),
('bar', 0);


-- insert 4 users
INSERT INTO users (ws_id, email, fullname, password_hash)
VALUES
(1, 'hedon@acme.com', 'Hedon', '$argon2id$v=19$m=19456,t=2,p=1$oAz92hafJDxT/KZeFUP1Rg$Mj2NpMdquq74Z/kOd3rqmep98XQmJwkgDSbIxU7qegc'),
(1, 'john@acme.com', 'John', '$argon2id$v=19$m=19456,t=2,p=1$oAz92hafJDxT/KZeFUP1Rg$Mj2NpMdquq74Z/kOd3rqmep98XQmJwkgDSbIxU7qegc'),
(1, 'jane@acme.com', 'Jane', '$argon2id$v=19$m=19456,t=2,p=1$oAz92hafJDxT/KZeFUP1Rg$Mj2NpMdquq74Z/kOd3rqmep98XQmJwkgDSbIxU7qegc'),
(1, 'joe@acme.com', 'Joe', '$argon2id$v=19$m=19456,t=2,p=1$oAz92hafJDxT/KZeFUP1Rg$Mj2NpMdquq74Z/kOd3rqmep98XQmJwkgDSbIxU7qegc'),
(1, 'jim@acme.com', 'Jim', '$argon2id$v=19$m=19456,t=2,p=1$oAz92hafJDxT/KZeFUP1Rg$Mj2NpMdquq74Z/kOd3rqmep98XQmJwkgDSbIxU7qegc');

-- insert 4 chats
-- insert public/private channel
INSERT INTO chats (ws_id, name, type, members)
VALUES
(1, 'general', 'public_channel', '{1,2,3,4,5}'),
(1, 'private', 'private_channel', '{1,2,3}');
-- insert unnamed chat
INSERT INTO chats(ws_id, type, members)
VALUES
(1, 'single', '{1,2}'),
(1, 'group', '{1,3,4}');