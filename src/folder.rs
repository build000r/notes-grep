use std::collections::{HashMap, HashSet};

use crate::notes::{FolderEntry, NgError};

#[derive(Debug, Clone)]
pub(crate) struct RawFolder {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) parent_id: Option<i64>,
    pub(crate) account_id: Option<i64>,
    pub(crate) account: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedTarget {
    pub(crate) parent_id: Option<i64>,
    pub(crate) account_id: Option<i64>,
    pub(crate) name: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone)]
struct ResolvedParent {
    id: Option<i64>,
    account_id: Option<i64>,
    path: Option<String>,
}

pub(crate) fn build_folder_entries(raw: &HashMap<i64, RawFolder>) -> Vec<FolderEntry> {
    let mut memo = HashMap::new();
    let mut entries = raw
        .values()
        .map(|folder| {
            let path = folder_path(folder.id, raw, &mut memo, &mut HashSet::new());
            let account_label = account_label(folder);
            let account_path = format!("{account_label}/{path}");
            FolderEntry {
                id: folder.id,
                name: folder.name.clone(),
                parent_id: folder.parent_id,
                account_id: folder.account_id,
                account: folder.account.clone(),
                path,
                account_path,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by_cached_key(|entry| entry.account_path.to_lowercase());
    entries
}

fn folder_path(
    id: i64,
    raw: &HashMap<i64, RawFolder>,
    memo: &mut HashMap<i64, String>,
    stack: &mut HashSet<i64>,
) -> String {
    if let Some(path) = memo.get(&id) {
        return path.clone();
    }

    let Some(folder) = raw.get(&id) else {
        return id.to_string();
    };
    if !stack.insert(id) {
        return folder.name.clone();
    }

    let path = folder
        .parent_id
        .and_then(|parent_id| raw.get(&parent_id).map(|_| parent_id))
        .map(|parent_id| {
            format!(
                "{}/{}",
                folder_path(parent_id, raw, memo, stack),
                folder.name
            )
        })
        .unwrap_or_else(|| folder.name.clone());
    stack.remove(&id);
    memo.insert(id, path.clone());
    path
}

fn account_label(folder: &RawFolder) -> String {
    folder
        .account
        .clone()
        .filter(|account| !account.trim().is_empty())
        .or_else(|| folder.account_id.map(|id| format!("account:{id}")))
        .unwrap_or_else(|| "unknown-account".to_owned())
}

pub(crate) fn normalize_path_arg(path: &str) -> Result<String, NgError> {
    let path = path.trim().trim_matches('/');
    let parts = path
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return Err(NgError::Command("folder path cannot be empty".to_owned()));
    }
    Ok(parts.join("/"))
}

fn split_path_arg(path: &str) -> Result<Vec<String>, NgError> {
    Ok(normalize_path_arg(path)?
        .split('/')
        .map(ToOwned::to_owned)
        .collect())
}

pub(crate) fn resolve_folder(folders: &[FolderEntry], path: &str) -> Result<FolderEntry, NgError> {
    let path = normalize_path_arg(path)?;
    let matches = folders
        .iter()
        .filter(|folder| folder.path == path || folder.account_path == path)
        .cloned()
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [folder] => Ok(folder.clone()),
        [] => Err(NgError::Command(format!("folder path not found: '{path}'"))),
        _ => Err(NgError::Command(format!(
            "folder path is ambiguous: '{path}'. Use an account-prefixed path such as '{}'.",
            matches[0].account_path
        ))),
    }
}

pub(crate) fn resolve_target(
    folders: &[FolderEntry],
    source: &FolderEntry,
    target_path: &str,
) -> Result<ResolvedTarget, NgError> {
    let parts = split_path_arg(target_path)?;
    let Some(name) = parts.last().cloned() else {
        return Err(NgError::Command(
            "target folder path cannot be empty".to_owned(),
        ));
    };
    if name.contains(':') && name.starts_with("account:") {
        return Err(NgError::Command(
            "target folder name cannot be an account placeholder".to_owned(),
        ));
    }

    let parent_parts = &parts[..parts.len() - 1];
    let parent = if parent_parts.is_empty() {
        ResolvedParent {
            id: source.parent_id,
            account_id: source.account_id,
            path: parent_path_for_source(folders, source),
        }
    } else {
        resolve_target_parent(folders, source, parent_parts)?
    };
    let path = match parent.path {
        Some(parent_path) => format!("{parent_path}/{name}"),
        None => name.clone(),
    };

    Ok(ResolvedTarget {
        parent_id: parent.id,
        account_id: parent.account_id,
        name,
        path,
    })
}

fn parent_path_for_source(folders: &[FolderEntry], source: &FolderEntry) -> Option<String> {
    source
        .parent_id
        .and_then(|parent_id| {
            folders
                .iter()
                .find(|folder| folder.id == parent_id)
                .map(|folder| folder.account_path.clone())
        })
        .or_else(|| Some(folder_account_label(source)))
}

fn resolve_target_parent(
    folders: &[FolderEntry],
    source: &FolderEntry,
    parent_parts: &[String],
) -> Result<ResolvedParent, NgError> {
    let parent_path = parent_parts.join("/");

    if let Ok(parent) = resolve_folder(folders, &parent_path) {
        return Ok(ResolvedParent {
            id: Some(parent.id),
            account_id: parent.account_id,
            path: Some(parent.account_path),
        });
    }

    let mut seen_accounts = HashSet::new();
    let accounts = folders
        .iter()
        .filter_map(|folder| {
            let key = (folder_account_label(folder), folder.account_id);
            if seen_accounts.insert(key.clone()) {
                Some(key)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let account_matches = accounts
        .iter()
        .filter(|(account, _)| account == &parent_path)
        .collect::<Vec<_>>();

    match account_matches.as_slice() {
        [(_, account_id)] => Ok(ResolvedParent {
            id: None,
            account_id: *account_id,
            path: Some(parent_path),
        }),
        [] if parent_path == "." => Ok(ResolvedParent {
            id: source.parent_id,
            account_id: source.account_id,
            path: None,
        }),
        [] => Err(NgError::Command(format!(
            "target parent folder path not found: '{parent_path}'"
        ))),
        _ => Err(NgError::Command(format!(
            "target parent account is ambiguous: '{parent_path}'"
        ))),
    }
}

pub(crate) fn parent_chain_contains(
    raw: &HashMap<i64, RawFolder>,
    mut parent_id: Option<i64>,
    needle: i64,
) -> bool {
    let mut seen = HashSet::new();
    while let Some(id) = parent_id {
        if id == needle {
            return true;
        }
        if !seen.insert(id) {
            return false;
        }
        parent_id = raw.get(&id).and_then(|folder| folder.parent_id);
    }
    false
}

pub(crate) fn descendant_folder_ids(raw: &HashMap<i64, RawFolder>, source_id: i64) -> Vec<i64> {
    // Apple Notes shouldn't produce parent cycles, but iCloud merge artifacts
    // and corrupt stores can. `seen` keeps a cycle from spinning this DFS into
    // an unbounded loop and an unbounded `Vec` (which would later blow up
    // `count_notes_in_folders`'s SQL parameter list). The traversal still
    // visits each reachable descendant exactly once.
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(source_id);
    let mut stack = vec![source_id];
    while let Some(parent_id) = stack.pop() {
        for folder in raw
            .values()
            .filter(|folder| folder.parent_id == Some(parent_id))
        {
            if seen.insert(folder.id) {
                out.push(folder.id);
                stack.push(folder.id);
            }
        }
    }
    out
}

pub(crate) fn sibling_exists(
    folders: &[FolderEntry],
    source_id: i64,
    account_id: Option<i64>,
    parent_id: Option<i64>,
    name: &str,
) -> bool {
    folders.iter().any(|folder| {
        folder.id != source_id
            && folder.account_id == account_id
            && folder.parent_id == parent_id
            && folder.name.eq_ignore_ascii_case(name)
    })
}

fn folder_account_label(folder: &FolderEntry) -> String {
    folder
        .account
        .clone()
        .filter(|account| !account.trim().is_empty())
        .or_else(|| folder.account_id.map(|id| format!("account:{id}")))
        .unwrap_or_else(|| "unknown-account".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_folder_entries_sorts_case_insensitively_by_account_path() {
        let raw = HashMap::from([
            (
                1,
                RawFolder {
                    id: 1,
                    name: "beta".into(),
                    parent_id: None,
                    account_id: Some(99),
                    account: Some("iCloud".into()),
                },
            ),
            (
                2,
                RawFolder {
                    id: 2,
                    name: "Alpha".into(),
                    parent_id: None,
                    account_id: Some(99),
                    account: Some("iCloud".into()),
                },
            ),
            (
                3,
                RawFolder {
                    id: 3,
                    name: "zeta".into(),
                    parent_id: None,
                    account_id: Some(12),
                    account: Some("Local".into()),
                },
            ),
        ]);

        let paths = build_folder_entries(&raw)
            .into_iter()
            .map(|entry| entry.account_path)
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["iCloud/Alpha", "iCloud/beta", "Local/zeta"]);
    }

    /// Regression: a parent cycle in the folder table (e.g. corrupt iCloud
    /// merge state where folder A.parent == B and B.parent == A) used to make
    /// `descendant_folder_ids` loop forever and grow an unbounded Vec, which
    /// would then explode `count_notes_in_folders`'s IN-clause. Cycles must
    /// terminate the traversal cleanly.
    #[test]
    fn descendant_folder_ids_terminates_on_cyclic_parent_chain() {
        let mut raw = HashMap::new();
        raw.insert(
            1,
            RawFolder {
                id: 1,
                name: "A".into(),
                parent_id: Some(2),
                account_id: Some(99),
                account: Some("iCloud".into()),
            },
        );
        raw.insert(
            2,
            RawFolder {
                id: 2,
                name: "B".into(),
                parent_id: Some(1),
                account_id: Some(99),
                account: Some("iCloud".into()),
            },
        );

        let descendants = descendant_folder_ids(&raw, 1);
        assert_eq!(descendants, vec![2], "cycle must visit B exactly once");

        let descendants = descendant_folder_ids(&raw, 2);
        assert_eq!(descendants, vec![1], "cycle must visit A exactly once");
    }
}
