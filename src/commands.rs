use std::collections::HashMap;

pub struct CommandTree {
    children: HashMap<&'static str, CommandTree>,
    flags: &'static [&'static str],
}

impl CommandTree {
    fn new(flags: &'static [&'static str]) -> Self {
        Self { children: HashMap::new(), flags }
    }

    fn child(mut self, name: &'static str, node: CommandTree) -> Self {
        self.children.insert(name, node);
        self
    }

    pub fn completions(&self, tokens: &[&str]) -> Vec<String> {
        if tokens.is_empty() {
            return self.all_options();
        }
        let (head, rest) = tokens.split_first().unwrap();
        if rest.is_empty() {
            // Partial match on current token
            self.matching_options(head)
        } else if let Some(child) = self.children.get(head) {
            child.completions(rest)
        } else {
            vec![]
        }
    }

    fn all_options(&self) -> Vec<String> {
        let mut opts: Vec<String> = self.children.keys().map(|s| s.to_string()).collect();
        opts.extend(self.flags.iter().map(|s| s.to_string()));
        opts.sort();
        opts
    }

    fn matching_options(&self, prefix: &str) -> Vec<String> {
        self.all_options()
            .into_iter()
            .filter(|s| s.starts_with(prefix))
            .collect()
    }
}

static GLOBAL_FLAGS: &[&str] = &[
    "--repository", "--ignore-working-copy", "--ignore-immutable",
    "--at-operation", "--debug", "--color", "--quiet", "--no-pager", "--config",
];

static LOG_FLAGS: &[&str] = &[
    "--revision", "--limit", "--reversed", "--no-graph", "--template",
    "--patch", "--summary", "--stat", "--git", "--color-words", "--context",
    "--name-only", "--types", "--count",
];

static DIFF_FLAGS: &[&str] = &[
    "--revision", "--from", "--to", "--summary", "--stat", "--types",
    "--name-only", "--git", "--color-words", "--tool", "--context",
    "--ignore-all-space", "--ignore-space-change",
];

static SHOW_FLAGS: &[&str] = &[
    "--template", "--summary", "--stat", "--types", "--name-only",
    "--git", "--color-words", "--tool", "--context", "--no-patch",
];

static DESCRIBE_FLAGS: &[&str] = &["--message", "--editor", "--reset-author"];
static NEW_FLAGS: &[&str] = &["--message", "--no-edit", "--insert-after", "--insert-before"];
static REBASE_FLAGS: &[&str] = &[
    "--branch", "--source", "--revision", "--onto",
    "--insert-after", "--insert-before", "--skip-emptied",
];
static SQUASH_FLAGS: &[&str] = &[
    "--revision", "--from", "--into", "--message",
    "--use-destination-message", "--interactive", "--tool", "--keep-emptied",
];
static SPLIT_FLAGS: &[&str] = &["--revision", "--parallel", "--interactive", "--tool"];
static COMMIT_FLAGS: &[&str] = &["--interactive", "--tool", "--message", "--editor"];
static BOOKMARK_LIST_FLAGS: &[&str] = &[
    "--all-remotes", "--tracked", "--conflicted", "--revision", "--template", "--sort",
];
static GIT_PUSH_FLAGS: &[&str] = &[
    "--remote", "--bookmark", "--all", "--change", "--revision", "--dry-run",
];
static GIT_FETCH_FLAGS: &[&str] = &["--remote", "--branch", "--all-remotes"];
static GIT_CLONE_FLAGS: &[&str] = &["--remote", "--colocate", "--no-colocate", "--depth", "--branch"];
static EVOLOG_FLAGS: &[&str] = &["--revision", "--limit", "--reversed", "--no-graph", "--template", "--patch"];
static OP_LOG_FLAGS: &[&str] = &["--limit", "--reversed", "--no-graph", "--template", "--op-diff", "--patch"];
static BOOKMARK_MOVE_FLAGS: &[&str] = &["--from", "--to", "--allow-backwards"];
static BOOKMARK_CREATE_FLAGS: &[&str] = &["--revision"];
static BOOKMARK_SET_FLAGS: &[&str] = &["--revision", "--allow-backwards"];
static DUPLICATE_FLAGS: &[&str] = &["--onto", "--insert-after", "--insert-before"];

pub fn build_tree() -> CommandTree {
    CommandTree::new(GLOBAL_FLAGS)
        .child("abandon", CommandTree::new(&["--retain-bookmarks", "--restore-descendants"]))
        .child("absorb", CommandTree::new(&["--from", "--into"]))
        .child("arrange", CommandTree::new(&[]))
        .child("bisect", CommandTree::new(&[])
            .child("run", CommandTree::new(&["--range", "--find-good"])))
        .child("bookmark", CommandTree::new(&[])
            .child("advance", CommandTree::new(&["--to"]))
            .child("create", CommandTree::new(BOOKMARK_CREATE_FLAGS))
            .child("delete", CommandTree::new(&[]))
            .child("forget", CommandTree::new(&["--include-remotes"]))
            .child("list", CommandTree::new(BOOKMARK_LIST_FLAGS))
            .child("move", CommandTree::new(BOOKMARK_MOVE_FLAGS))
            .child("rename", CommandTree::new(&["--overwrite-existing"]))
            .child("set", CommandTree::new(BOOKMARK_SET_FLAGS))
            .child("track", CommandTree::new(&[]))
            .child("untrack", CommandTree::new(&[])))
        .child("commit", CommandTree::new(COMMIT_FLAGS))
        .child("config", CommandTree::new(&[])
            .child("edit", CommandTree::new(&["--user", "--repo", "--workspace"]))
            .child("get", CommandTree::new(&[]))
            .child("list", CommandTree::new(&["--include-defaults", "--include-overridden", "--user", "--repo", "--workspace", "--template"]))
            .child("path", CommandTree::new(&["--user", "--repo", "--workspace"]))
            .child("set", CommandTree::new(&["--user", "--repo", "--workspace"]))
            .child("unset", CommandTree::new(&["--user", "--repo", "--workspace"])))
        .child("describe", CommandTree::new(DESCRIBE_FLAGS))
        .child("diff", CommandTree::new(DIFF_FLAGS))
        .child("diffedit", CommandTree::new(&["--revision", "--from", "--to", "--tool", "--restore-descendants"]))
        .child("duplicate", CommandTree::new(DUPLICATE_FLAGS))
        .child("edit", CommandTree::new(&[]))
        .child("evolog", CommandTree::new(EVOLOG_FLAGS))
        .child("file", CommandTree::new(&[])
            .child("annotate", CommandTree::new(&["--revision", "--template"]))
            .child("chmod", CommandTree::new(&["--revision"]))
            .child("list", CommandTree::new(&["--revision", "--template"]))
            .child("search", CommandTree::new(&["--revision", "--pattern"]))
            .child("show", CommandTree::new(&["--revision", "--template"]))
            .child("track", CommandTree::new(&[]))
            .child("untrack", CommandTree::new(&[])))
        .child("fix", CommandTree::new(&["--source", "--include-unchanged-files"]))
        .child("gerrit", CommandTree::new(&[])
            .child("upload", CommandTree::new(&["--revision"])))
        .child("git", CommandTree::new(&[])
            .child("clone", CommandTree::new(GIT_CLONE_FLAGS))
            .child("colocation", CommandTree::new(&[])
                .child("disable", CommandTree::new(&[]))
                .child("enable", CommandTree::new(&[]))
                .child("status", CommandTree::new(&[])))
            .child("export", CommandTree::new(&[]))
            .child("fetch", CommandTree::new(GIT_FETCH_FLAGS))
            .child("import", CommandTree::new(&[]))
            .child("init", CommandTree::new(&["--colocate", "--no-colocate", "--git-repo"]))
            .child("push", CommandTree::new(GIT_PUSH_FLAGS))
            .child("remote", CommandTree::new(&[])
                .child("add", CommandTree::new(&["--fetch-tags", "--push-url"]))
                .child("list", CommandTree::new(&[]))
                .child("remove", CommandTree::new(&[]))
                .child("rename", CommandTree::new(&[]))
                .child("set-url", CommandTree::new(&["--push", "--fetch"])))
            .child("root", CommandTree::new(&[])))
        .child("help", CommandTree::new(&["--keyword"]))
        .child("interdiff", CommandTree::new(&["--from", "--to", "--summary", "--stat", "--git", "--color-words"]))
        .child("log", CommandTree::new(LOG_FLAGS))
        .child("metaedit", CommandTree::new(&["--update-change-id", "--message", "--update-author-timestamp"]))
        .child("new", CommandTree::new(NEW_FLAGS))
        .child("next", CommandTree::new(&["--edit", "--no-edit", "--conflict"]))
        .child("operation", CommandTree::new(&[])
            .child("abandon", CommandTree::new(&[]))
            .child("diff", CommandTree::new(&["--operation", "--from", "--to", "--no-graph", "--patch"]))
            .child("integrate", CommandTree::new(&[]))
            .child("log", CommandTree::new(OP_LOG_FLAGS))
            .child("restore", CommandTree::new(&["--what"]))
            .child("revert", CommandTree::new(&["--what"]))
            .child("show", CommandTree::new(&["--no-graph", "--template", "--patch", "--no-op-diff"])))
        .child("parallelize", CommandTree::new(&[]))
        .child("prev", CommandTree::new(&["--edit", "--no-edit", "--conflict"]))
        .child("rebase", CommandTree::new(REBASE_FLAGS))
        .child("redo", CommandTree::new(&[]))
        .child("resolve", CommandTree::new(&["--revision", "--list", "--tool"]))
        .child("restore", CommandTree::new(&["--from", "--into", "--changes-in", "--interactive", "--tool", "--restore-descendants"]))
        .child("revert", CommandTree::new(&["--revision", "--onto", "--insert-after", "--insert-before"]))
        .child("root", CommandTree::new(&[]))
        .child("show", CommandTree::new(SHOW_FLAGS))
        .child("sign", CommandTree::new(&["--revision", "--key"]))
        .child("simplify-parents", CommandTree::new(&["--source", "--revision"]))
        .child("sparse", CommandTree::new(&[])
            .child("edit", CommandTree::new(&[]))
            .child("list", CommandTree::new(&[]))
            .child("reset", CommandTree::new(&[]))
            .child("set", CommandTree::new(&["--add", "--remove", "--clear"])))
        .child("split", CommandTree::new(SPLIT_FLAGS))
        .child("squash", CommandTree::new(SQUASH_FLAGS))
        .child("status", CommandTree::new(&[]))
        .child("tag", CommandTree::new(&[])
            .child("delete", CommandTree::new(&[]))
            .child("list", CommandTree::new(&["--all-remotes", "--conflicted", "--revision", "--template", "--sort"]))
            .child("set", CommandTree::new(&["--revision", "--allow-move"])))
        .child("undo", CommandTree::new(&[]))
        .child("unsign", CommandTree::new(&["--revision"]))
        .child("util", CommandTree::new(&[])
            .child("completion", CommandTree::new(&[]))
            .child("config-schema", CommandTree::new(&[]))
            .child("exec", CommandTree::new(&[]))
            .child("gc", CommandTree::new(&["--expire"]))
            .child("install-man-pages", CommandTree::new(&[]))
            .child("markdown-help", CommandTree::new(&[]))
            .child("snapshot", CommandTree::new(&[])))
        .child("version", CommandTree::new(&[]))
        .child("workspace", CommandTree::new(&[])
            .child("add", CommandTree::new(&["--name", "--revision", "--message", "--sparse-patterns"]))
            .child("forget", CommandTree::new(&[]))
            .child("list", CommandTree::new(&["--template"]))
            .child("rename", CommandTree::new(&[]))
            .child("root", CommandTree::new(&["--name"]))
            .child("update-stale", CommandTree::new(&[])))
}

/// Check if the resolved command path expects bookmark names as arguments
pub fn expects_bookmark_arg(tokens: &[&str]) -> bool {
    matches!(
        tokens,
        ["bookmark", "advance" | "create" | "delete" | "forget" | "move" | "rename" | "set" | "track" | "untrack", ..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_completions() {
        let tree = build_tree();
        let c = tree.completions(&["b"]);
        assert!(c.contains(&"bookmark".to_string()));
        assert!(c.contains(&"bisect".to_string()));
    }

    #[test]
    fn nested_completions() {
        let tree = build_tree();
        let c = tree.completions(&["bookmark", ""]);
        assert!(c.contains(&"advance".to_string()));
        assert!(c.contains(&"delete".to_string()));
        assert!(c.contains(&"list".to_string()));
    }

    #[test]
    fn flag_completions() {
        let tree = build_tree();
        let c = tree.completions(&["log", "--l"]);
        assert!(c.contains(&"--limit".to_string()));
    }

    #[test]
    fn deep_nested() {
        let tree = build_tree();
        let c = tree.completions(&["git", "remote", ""]);
        assert!(c.contains(&"add".to_string()));
        assert!(c.contains(&"list".to_string()));
        assert!(c.contains(&"remove".to_string()));
    }

    #[test]
    fn empty_input() {
        let tree = build_tree();
        let c = tree.completions(&[""]);
        assert!(c.contains(&"log".to_string()));
        assert!(c.contains(&"status".to_string()));
        assert!(c.len() > 30);
    }

    #[test]
    fn bookmark_arg_detection() {
        assert!(expects_bookmark_arg(&["bookmark", "delete"]));
        assert!(expects_bookmark_arg(&["bookmark", "move", "main"]));
        assert!(!expects_bookmark_arg(&["bookmark", "list"]));
        assert!(!expects_bookmark_arg(&["log"]));
    }
}
