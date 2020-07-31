CREATE TABLE public.migration_patches (
    patch_name TEXT NOT NULL
    ,sha1_code TEXT NOT NULL
    ,created_at TIMESTAMPTZ NOT NULL
    ,PRIMARY KEY (patch_name)
);