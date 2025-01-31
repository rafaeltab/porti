CREATE TABLE "Organization" (
    id bigint primary key,
    name varchar
);

CREATE TABLE "PlatformAccount" (
    id bigint primary key,
    organization_id bigint references "Organization",
    name varchar,
    platform_name varchar
);
