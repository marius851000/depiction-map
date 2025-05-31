use anyhow::bail;
use git2::{ErrorCode, Repository, Signature};
use log::info;
use std::path::Path;

pub fn make_commit(repo: &Repository, path: &Path, message: &str) -> anyhow::Result<()> {
    let old_tree = repo.head()?.peel_to_tree()?;

    let mut index = repo.index()?;
    index.add_path(path)?;
    index.write()?;

    let new_tree_ref = index.write_tree()?;
    let new_tree = repo.find_tree(new_tree_ref)?;

    if new_tree.id() == old_tree.id() {
        info!("No change detected, not commiting.");
        return Ok(());
    }

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
        &new_tree,
        &parents,
    )?;

    Ok(())
}
