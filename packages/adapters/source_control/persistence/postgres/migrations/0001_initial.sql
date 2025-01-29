CREATE TABLE "Organization" (
    id integer primary key,
    name varchar
);

CREATE TABLE "PlatformAccount" (
    id integer primary key,
    organization_id integer references "Organization",
    name varchar,
    platform_name varchar
);
