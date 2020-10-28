use crate::error::SystemError;
use tokio_postgres::Client;
use std::{
    cmp::{Ordering, Ord, PartialOrd,},
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};


#[derive(Eq)]
pub struct FileEntry {
    pub file_name: String,
    pub sha: String,
    pub content: String,
}

impl FileEntry {
    pub fn new(entry: &std::fs::DirEntry) -> Result<Self, SystemError> {
        let path = entry.path();
        let file_name = match path.as_os_str().to_str() {
            Some(str) => str,
            None => return Err(SystemError::Other("file_name convert error".to_owned()))
        };
        let mut file = BufReader::new(std::fs::File::open(entry.path())?);
        let mut s = String::new();
        let _ = file.read_to_string(&mut s)?;
        Ok(FileEntry{
            file_name: file_name.to_owned(),
            sha: sha1::Sha1::from(&s).digest().to_string(),
            content: s,
        })
    }

    pub async fn is_exists(
        &self,
        client: &Client,
        sha1_flag: bool,
    ) -> Result<bool, SystemError> {
        let sql = r#"
            SELECT
                t1.sha1_code
            FROM
                public.migration_patches AS t1
            WHERE
                t1.patch_name = $1
        "#;
        let rows = client
            .query(sql, &[&self.file_name])
            .await?;
        if rows.len() == 0 {
            return Ok(false);
        } else if !sha1_flag {
            return Ok(true);
        }
        let sha1_code: &str = rows[0].get(0);
        Ok(self.sha == sha1_code)
    }

    pub fn write(&self, patch_file: &mut BufWriter<File>) -> Result<(), SystemError> {
        patch_file.write(&self.content.as_bytes())?;
        patch_file.write("\n\n".as_bytes())?;
        patch_file.write(&add(&self.file_name, &self.sha).as_bytes())?;
        Ok(())
    }
}

fn add(
    patch_name: &str,
    sha1_code: &str,
) -> String {
    format!(r#"
    INSERT INTO public.migration_patches (
        patch_name
        ,sha1_code
        ,created_at
        ,updated_at
    ) VALUES (
        '{}'
        ,'{}'
        ,NOW()
        ,NOW()
    )
    ON CONFLICT (patch_name) DO UPDATE SET
        sha1_code = EXCLUDED.sha1_code
        ,updated_at = EXCLUDED.updated_at;

"#, patch_name, sha1_code)
}

impl PartialEq for FileEntry {
    fn eq(&self, other: &FileEntry) -> bool {
        self.file_name == other.file_name
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &FileEntry) -> Option<Ordering> {
        self.file_name.partial_cmp(&other.file_name)
    }
}


impl Ord for FileEntry {
    fn cmp(&self, other: &FileEntry) -> Ordering {
        self.file_name.cmp(&other.file_name)
    }
}
