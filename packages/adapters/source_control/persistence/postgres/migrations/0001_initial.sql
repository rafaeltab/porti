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

CREATE INDEX IF NOT EXISTS "PlatformAccount_organization_id_IDX"
    ON public."PlatformAccount" USING btree
    (organization_id ASC NULLS LAST)
    WITH (deduplicate_items=False)
    TABLESPACE pg_default;
