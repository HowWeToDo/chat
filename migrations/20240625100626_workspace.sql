-- Add migration script here

-- workspace for users
create table if not exists workspaces (
    id bigserial primary key,
    name varchar(32) not null unique,
    owner_id bigint not null references users(id),
    created_at timestamptz default current_timestamp
);

-- alter users table to add ws_id
alter table users
    add column ws_id bigint references workspaces(id);

-- add super user 0 and workspace 0
begin;
insert into users(id, fullname, email, password_hash)
    values(0, 'super user', 'super@none.org', '');
insert into workspaces(id, name, owner_id)
    values (0, 'none', 0);
update users set ws_id = 0 where id = 0;
commit;

-- alter user table to make ws_id not null
alter table users
    alter column ws_id set not null;
