use std::path::Path;

use anyhow::bail;
use git2::{ErrorCode, Repository, Signature};

pub fn make_commit(repo: &Repository, path: &Path, message: &str) -> anyhow::Result<()> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    index.write()?;

    let tree_oid = index.write_tree()?;

    let tree = repo.find_tree(tree_oid)?;
    let head = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(err) => {
            if err.code() == ErrorCode::UnbornBranch {
                None
            } else {
                bail!(err);
            }
        }
    };

    let mut parents = Vec::new();
    if let Some(head) = head.as_ref() {
        parents.push(head);
    };

    let signature = Signature::now("depict_bot", "nomail@example.org")?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )?;

    Ok(())
}
