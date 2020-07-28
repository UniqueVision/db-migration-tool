CREATE TABLE public.migration_stored_procedures (
    file_path_code TEXT NOT NULL
    ,sha1_count BIGINT NOT NULL
    ,created_at TIMESTAMPTZ NOT NULL
    ,updated_at TIMESTAMPTZ NOT NULL
    ,PRIMARY KEY (file_path_code)
);