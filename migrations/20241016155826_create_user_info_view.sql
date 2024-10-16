CREATE VIEW user_info_view AS
    SELECT users.displayname, users.handle, users.user_id
    FROM users;
