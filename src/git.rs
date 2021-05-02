use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct Git {
    executable: String,
    working_dir: PathBuf,
    dry: bool,
}

#[derive(Debug, Clone)]
pub struct CommitDetails {
    commit_sha: String,
    subject: String,
    description: String,
    branch_name: String,
}

impl CommitDetails {
    pub fn from_str(commit_branch_string: &str) -> CommitDetails {
        let split_str: Vec<&str> = commit_branch_string.split("|").collect();
        CommitDetails {
            commit_sha: split_str[0].to_string(),
            subject: split_str[1].to_string(),
            description: split_str[2].to_string(),
            branch_name: split_str[3].to_string(),
        }
    }
}

impl std::fmt::Display for CommitDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.commit_sha, self.subject, self.description, self.branch_name
        )
    }
}

impl Git {
    pub fn new(executable: Option<String>, path: Option<PathBuf>, dry: bool) -> Git {
        let git = Git {
            executable: executable.unwrap_or("git".to_string()),
            working_dir: path.unwrap_or(env::current_dir().unwrap()),
            dry: dry,
        };
        git.check_valid_state();
        git
    }

    fn check_valid_state(&self) {
        // check that is valid git repo
        self.execute(vec!["rev-parse", "--git-dir"])
            .expect("Not a git repository!");
    }

    fn create_branch(&self, name: &str, from: &str) {
        let current_branch = self.get_current_branch();
        if self.dry {
            println!("git checkout -b {} {}", name, from);
            return;
        }
        self.execute(vec!["checkout", "-b", name, from]).ok();
        self.switch(&current_branch);
    }

    fn execute(&self, args: Vec<&str>) -> Result<String, i32> {
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

    fn get_default_branch(&self) -> String {
        let branch_regex = Regex::new(r"(HEAD\sbranch:\s)(.*)").unwrap();
        let remote = self.get_remote();
        let remote_output_str = self.execute(vec!["remote", "show", &remote]).ok().unwrap();
        let remote_output_split: Vec<&str> = remote_output_str
            .lines()
            .filter(|s| branch_regex.is_match(s))
            .collect();
        let first_entry = remote_output_split.first().unwrap();
        branch_regex
            .captures(first_entry)
            .unwrap()
            .get(2)
            .unwrap()
            .as_str()
            .to_owned()
    }

    fn get_current_branch(&self) -> String {
        self.execute(vec!["rev-parse", "--abbrev-ref", "HEAD"])
            .ok()
            .unwrap()
    }

    fn get_remote(&self) -> String {
        // TODO: use "origin" by default and ask user for other source, if required
        // This choice should then be remembered and saved somewhere
        String::from("origin")
    }

    fn switch(&self, branch_name: &str) {
        if self.dry {
            println!("git switch {}", branch_name);
            return;
        }
        self.execute(vec!["switch", &branch_name]).ok();
    }

    fn rebase(&self, commit_sha: &str, rebase_branch: &str) -> Result<String, i32> {
        if self.dry {
            println!("git rebase {} {}", commit_sha, rebase_branch);
            return Ok("".to_string());
        }
        self.execute(vec!["rebase", commit_sha, rebase_branch])
    }

    fn push(&self, branch_name: &str, push_options: &Vec<String>) {
        if self.dry {
            println!("git push {}", branch_name);
            return;
        }
        let remote = self.get_remote();
        let mut push_command = Vec::new();
        push_command.push("push");
        // TODO: this should be possible with .append()
        for option in push_options {
            push_command.push(&option);
        }
        push_command.push("-u");
        push_command.push(&remote);
        push_command.push(branch_name);
        self.execute(push_command).ok();
    }
}

pub fn get_commit_branch_till_branch(git: &Git, branch_name: &str) -> Vec<CommitDetails> {
    git.execute(vec![
        "rev-list",
        "--format=%H|%s|%b|%f",
        "--no-merges",
        &format!("HEAD...{}", branch_name),
    ])
    .expect("Could not get commits!")
    .lines()
    .map(|s| s.to_string())
    .filter(|s| !s.starts_with("commit "))
    .map(|s| CommitDetails::from_str(&s))
    .collect()
}

fn create_branches(git: &Git, names: &Vec<String>, from_branch: &str) {
    for name in names {
        git.create_branch(name, from_branch);
    }
}

fn get_default_push_options(
    target_branch: &str,
    commit_message: &str,
    description: &str,
) -> Vec<String> {
    let mut options: Vec<String> = vec![
        "-o".to_string(),
        "merge_request.create".to_string(),
        "-o".to_string(),
        "merge_request.remove_source_branch".to_string(),
        "-o".to_string(),
        format!("merge_request.target={}", target_branch),
    ];
    if !commit_message.is_empty() {
        options.push("-o".to_string());
        options.push(format!("merge_request.title={}", commit_message));
    }
    if !description.is_empty() {
        options.push("-o".to_string());
        options.push(format!("merge_request.description={}", description));
    }
    options
}

fn push_branches(git: &Git, commit_details: &Vec<CommitDetails>, target_branch: &str) {
    let last_index = commit_details.len() - 1;
    for (i, commit_detail) in commit_details.iter().enumerate() {
        let subject = if i != last_index {
            format!("{} {}", "Draft: ", commit_detail.subject)
        } else {
            commit_detail.subject.to_string()
        };
        let push_options =
            get_default_push_options(target_branch, &subject, &commit_detail.description);
        git.push(&commit_detail.branch_name, &push_options);
    }
}

fn rebase_commits_onto_branches(git: &Git, commit_details: &Vec<CommitDetails>) {
    // ignore errors
    for commit_branch in commit_details {
        println!("Created {}", commit_branch);
        git.rebase(&commit_branch.commit_sha, &commit_branch.branch_name)
            .ok();
    }
}

fn get_branches(commit_details: &Vec<CommitDetails>) -> Vec<String> {
    commit_details
        .clone()
        .into_iter()
        .map(|cb| cb.branch_name)
        .collect()
}

pub fn create_separate_merge_requests(git: &Git) {
    let default_branch = git.get_default_branch();
    let commit_details = get_commit_branch_till_branch(&git, &default_branch);
    let branches: Vec<String> = get_branches(&commit_details);
    create_branches(&git, &branches, &default_branch);
    rebase_commits_onto_branches(&git, &commit_details);
    push_branches(&git, &commit_details, &default_branch);
}
