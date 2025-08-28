use std::path::Path;

use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt as _,
};

pub async fn append_to_mod_file(
    path: impl AsRef<Path>,
    contents: &str,
) -> Result<(), anyhow::Error> {
    let file_contents = fs::read_to_string(path.as_ref()).await?;
    let file_contents = file_contents.trim();

    let mut options = OpenOptions::new();
    options.write(true);

    if file_contents.is_empty() {
        options.truncate(true);
    } else {
        options.append(true);
    }

    let mut file = options.open(path.as_ref()).await?;
    for ele in contents.split("\n") {
        if file_contents.contains(ele) {
            continue;
        }
        file.write_all(ele.as_bytes()).await?;
    }

    Ok(())
}
