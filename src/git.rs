use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str::Lines;

pub struct Git {
    executable: String,
    working_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CommitBranch {
    commit_sha: String,
    branch_name: String,
}

impl CommitBranch {
    pub fn from_str(commit_branch_string: &str) -> CommitBranch {
        let split_str: Vec<&str> = commit_branch_string.split("|").collect();
        CommitBranch {
            commit_sha: split_str[0].to_string(),
            branch_name: split_str[1].to_string(),
        }
    }
}

impl std::fmt::Display for CommitBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.commit_sha, self.branch_name)
    }
}

impl Git {
    pub fn new(executable: Option<String>, path: Option<PathBuf>) -> Git {
        let git = Git {
            executable: executable.unwrap_or("git".to_string()),
            working_dir: path.unwrap_or(env::current_dir().unwrap()),
        };
        git.check_valid_state();
        git
    }

    fn check_valid_state(&self) {
        // check that is valid git repo
        self.execute(vec!["rev-parse", "--git-dir"])
            .expect("Not a git repository!");
    }

    fn create_branch(&self, name: &str) {
        // ignore error for now
        self.execute(vec!["branch", name]).ok();
    }

    pub fn execute(&self, args: Vec<&str>) -> Result<String, i32> {
        let output = Command::new(self.executable.clone())
            .current_dir(self.working_dir.as_path())
            .args(args)
            .output()
            .unwrap();
        if output.status.code().unwrap() != 0 {
            return Err(1);
        }
        Ok(String::from_utf8(output.stdout).unwrap())
    }

    pub fn get_commit_branch_till_branch(&self, branch_name: String) -> Vec<CommitBranch> {
        self.execute(vec![
            "rev-list",
            "--format=%H|%f",
            "--no-merges",
            &format!("HEAD...{}", &branch_name),
        ])
        .expect("Could not get commits!")
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.starts_with("commit "))
        .map(|s| CommitBranch::from_str(&s))
        .collect()
    }

    fn get_default_branch(&self) -> Option<String> {
        Some(String::from("master"))
    }

    fn get_current_branch(&self) -> String {
        self.execute(vec!["rev-parse", "--abbrev-ref", "HEAD"])
            .ok()
            .unwrap()
    }

    fn get_remote(&self) -> Option<String> {
        // TODO: use "origin" by default and ask user for other source, if required
        // This choice should then be remembered and saved somewhere
        Some(String::from("origin"))
    }

    fn push(&self, branch_name: &str) {}
}

fn create_branches(git: &Git, names: &Vec<String>) {
    for name in names {
        git.create_branch(name);
    }
}

fn push_branches(git: &Git, branches: &Vec<String>) {
    for branch in branches {
        git.push(branch);
    }
}

fn rebase_commits_onto_branches(git: &Git, commits_and_branches: &Vec<CommitBranch>) {
    // ignore errors
    let current_branch = git.get_current_branch();
    for commit_branch in commits_and_branches {
        git.execute(vec!["switch", &commit_branch.branch_name]).ok();
        println!("Created {}", commit_branch);
        git.execute(vec!["rebase", &commit_branch.commit_sha]).ok();
    }
    git.execute(vec!["switch", &current_branch]).ok();
}

pub fn create_separate_merge_requests(git: &Git) {
    let default_branch = git.get_default_branch().unwrap();
    let commits_and_branches = git.get_commit_branch_till_branch(default_branch);
    let branches: Vec<String> = commits_and_branches
        .clone()
        .into_iter()
        .map(|cb| cb.branch_name)
        .collect();
    let branches = create_branches(&git, &branches);
    rebase_commits_onto_branches(&git, &commits_and_branches);
    //push_branches(&git, &branches);
    // get default branch
    // get commits that differ from main on current branch
    // make separate branches -> name from commit message
    // push separate branches with pushoptions to create MR
}
