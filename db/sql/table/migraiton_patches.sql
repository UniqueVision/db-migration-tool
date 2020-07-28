CREATE TABLE public.migration_patches (
    patch_name TEXT NOT NULL
    ,patch_content TEXT NOT NULL
    ,created_at TIMESTAMPTZ NOT NULL
    ,PRIMARY KEY (patch_name)
);