create table teams (
    "id" serial primary key,
    "url" text not null,
    "name" text not null,
    "enabled" boolean not null default true,
    unique (url)
);
