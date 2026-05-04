PRAGMA foreign_keys = ON;

INSERT INTO users(id, username, email, password_hash, created_at) VALUES
  ('11111111-1111-1111-1111-111111111111', 'admin', 'admin@poseidon.local', '$argon2id$v=19$m=4096,t=3,p=1$cG9zZWlkb24tYWRtaW4tc2FsdA$ZOutjAqQUlZM6QrcTboStnxE3ZeOX6oj6QRwFF4E+0g', 1735689600),
  ('22222222-2222-2222-2222-222222222222', 'alice', 'alice@poseidon.local', '$argon2id$v=19$m=4096,t=3,p=1$cG9zZWlkb24tZGVtby1zYWx0$KP6lrMGAIQvf32hFhG/x6g0ZXprV5FFPlPxeJZ4YsCk', 1735689660),
  ('33333333-3333-3333-3333-333333333333', 'bob', 'bob@poseidon.local', '$argon2id$v=19$m=4096,t=3,p=1$cG9zZWlkb24tZGVtby1zYWx0$KP6lrMGAIQvf32hFhG/x6g0ZXprV5FFPlPxeJZ4YsCk', 1735689720);

INSERT INTO friendships(user_a, user_b, created_at) VALUES
  ('11111111-1111-1111-1111-111111111111', '22222222-2222-2222-2222-222222222222', 1735690200),
  ('22222222-2222-2222-2222-222222222222', '33333333-3333-3333-3333-333333333333', 1735690260);

INSERT INTO friend_requests(id, from_user_id, to_user_id, created_at) VALUES
  ('44444444-4444-4444-4444-444444444444', '33333333-3333-3333-3333-333333333333', '11111111-1111-1111-1111-111111111111', 1735690320);

INSERT INTO servers(id, name, description, owner_id, is_public, created_at) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Poseidon Hub', 'Közös bemutató szerver a projekt demó tartalmához.', '11111111-1111-1111-1111-111111111111', 1, 1735690400),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Study Group', 'Privát szerver a meghívás alapú hozzáféréshez.', '22222222-2222-2222-2222-222222222222', 0, 1735690460);

INSERT INTO server_members(server_id, user_id, role, joined_at) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'owner', 1735690400),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '22222222-2222-2222-2222-222222222222', 'user', 1735690500),
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '33333333-3333-3333-3333-333333333333', 'moderator', 1735690560),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '22222222-2222-2222-2222-222222222222', 'owner', 1735690460);

INSERT INTO server_invites(code, server_id, created_by, created_at) VALUES
  ('POSEIDON-DEMO-HUB', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 1735690600);

INSERT INTO server_user_invites(id, server_id, from_user_id, to_user_id, created_at) VALUES
  ('55555555-5555-5555-5555-555555555555', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '22222222-2222-2222-2222-222222222222', '33333333-3333-3333-3333-333333333333', 1735690660);

INSERT INTO channels(id, server_id, name, emoji, created_at) VALUES
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'general', '💬', 1735690720),
  ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'random', '✨', 1735690780);

INSERT INTO channel_messages(id, channel_id, user_id, content, created_at) VALUES
  ('66666666-6666-6666-6666-666666666666', 'cccccccc-cccc-cccc-cccc-cccccccccccc', '11111111-1111-1111-1111-111111111111', 'Üdv a Poseidon demó szerverén!', 1735690840),
  ('77777777-7777-7777-7777-777777777777', 'cccccccc-cccc-cccc-cccc-cccccccccccc', '22222222-2222-2222-2222-222222222222', 'A CRUD műveletek itt kipróbálhatók.', 1735690900),
  ('88888888-8888-8888-8888-888888888888', 'dddddddd-dddd-dddd-dddd-dddddddddddd', '33333333-3333-3333-3333-333333333333', 'Random csatorna a bemutatóhoz.', 1735690960);

INSERT INTO dm_threads(id, user_a, user_b, created_at) VALUES
  ('99999999-9999-9999-9999-999999999999', '22222222-2222-2222-2222-222222222222', '33333333-3333-3333-3333-333333333333', 1735691020);

INSERT INTO dm_messages(id, thread_id, user_id, content, created_at) VALUES
  ('aaaaaaaa-1111-1111-1111-111111111111', '99999999-9999-9999-9999-999999999999', '22222222-2222-2222-2222-222222222222', 'Szia Bob, működik a DM szál?', 1735691080),
  ('bbbbbbbb-1111-1111-1111-111111111111', '99999999-9999-9999-9999-999999999999', '33333333-3333-3333-3333-333333333333', 'Igen, és a teljes API is elérhető.', 1735691140);