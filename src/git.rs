use regex::Regex;
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
    branch_name: String,
}

impl CommitDetails {
    pub fn from_str(commit_branch_string: &str) -> CommitDetails {
        let split_str: Vec<&str> = commit_branch_string.split('|').collect();
        CommitDetails {
            commit_sha: split_str[0].to_string(),
            subject: split_str[1].to_string(),
            branch_name: split_str[2].to_string(),
        }
    }
}

impl std::fmt::Display for CommitDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.commit_sha, self.subject, self.branch_name
        )
    }
}

impl Git {
    pub fn new(executable: Option<String>, path: PathBuf, dry: bool) -> Git {
        let git = Git {
            executable: executable.unwrap_or_else(|| "git".to_string()),
            working_dir: path,
            dry,
        };
        git.check_valid_state();
        git
    }

    fn check_valid_state(&self) {
        if !self.working_dir.exists() {
            panic!("Not a valid working directory!");
        }
        self.execute(vec!["rev-parse", "--git-dir"])
            .expect("Not a valid git repository!");
    }

    fn create_branch(&self, name: &str, from: &str) {
        if self.dry {
            println!("git checkout -b {} {}", name, from);
            return;
        }
        self.execute(vec!["checkout", "-b", name, from])
            .expect("Could not create branch");
    }

    fn execute(&self, args: Vec<&str>) -> Result<String, i32> {
        let output = Command::new(self.executable.clone())
            .current_dir(self.working_dir.as_path())
            .args(&args)
            .output()
            .expect("Could not execute git command");
        let status_code = output.status.code().expect("Failed to get status");
        if status_code != 0 {
            println!("{:?}", &String::from_utf8(output.stderr).ok());
            return Err(status_code);
        }
        Ok(String::from_utf8(output.stdout).expect("Could not parse output"))
    }

    fn get_default_branch(&self) -> String {
        let branch_regex = Regex::new(r"(HEAD\sbranch:\s)(.*)").expect("Couldn't compile regex");
        let remote = self.get_remote();
        let remote_output_str = self
            .execute(vec!["remote", "show", &remote])
            .expect("Could not show remote! Maybe you haven't added any yet?");
        let remote_output_split: Vec<&str> = remote_output_str
            .lines()
            .filter(|s| branch_regex.is_match(s))
            .collect();
        let first_entry = remote_output_split
            .first()
            .expect("Couldn't find regex match!");
        branch_regex
            .captures(first_entry)
            .expect("No captures found!")
            .get(2)
            .expect("Couldn't find second match group")
            .as_str()
            .to_owned()
    }

    fn get_remote(&self) -> String {
        // TODO: use "origin" by default and ask user for other source, if required
        // This choice should then be remembered and saved somewhere
        String::from("origin")
    }

    fn rebase(&self, commit_sha: &str, rebase_branch: &str) {
        if self.dry {
            println!("git rebase {} {}", commit_sha, rebase_branch);
            return;
        }
        self.execute(vec!["rebase", commit_sha, rebase_branch])
            .expect("Rebase failed!");
    }

    fn push(&self, branch_name: &str, push_options: &[String]) {
        if self.dry {
            println!("git push {}", branch_name);
            return;
        }
        let remote = self.get_remote();
        let mut push_command = vec!["push"];
        // TODO: this should be possible with .append()
        for option in push_options {
            push_command.push(&option);
        }
        push_command.push("-u");
        push_command.push(&remote);
        push_command.push(branch_name);
        self.execute(push_command)
            .expect("Could not push to remote");
    }

    fn cherry_pick(&self, original_branch: &str, commit_sha: &str, branch_name: &str) {
        self.switch(&branch_name);
        if self.dry {
            println!("git cherry-pick {}", &commit_sha);
        } else {
            self.execute(vec!["cherry-pick", &commit_sha])
                .expect("Could not cherry pick commits");
        }
        self.switch(&original_branch);
    }

    fn get_current_branch(&self) -> String {
        self.execute(vec!["rev-parse", "--abbrev-ref", "HEAD"])
            .expect("Could not get current branch!")
            .trim()
            .to_owned()
    }

    fn switch(&self, branch_name: &str) {
        if self.get_current_branch() == branch_name {
            return;
        }
        if self.dry {
            println!("git switch {}", branch_name);
            return;
        }
        self.execute(vec!["switch", &branch_name])
            .expect("Unable to switch branch");
    }

    fn hard_reset(&self) {
        let remote_branch = format!("{}/{}", self.get_remote(), self.get_current_branch());
        if self.dry {
            println!("git reset --hard {}", &remote_branch);
            return;
        }
        self.execute(vec!["reset", "--hard", &remote_branch])
            .expect("Unable to reset branch");
        println!("Reset branch to {}", &remote_branch);
    }
}

pub fn get_commit_branch_till_branch(
    git: &Git,
    branch_name: &str,
    remote: &str,
) -> Vec<CommitDetails> {
    git.execute(vec![
        "rev-list",
        "--format=%H|%s|%f",
        "--no-merges",
        &format!("HEAD...{}/{}", remote, branch_name),
    ])
    .expect("Could not get commits!")
    .lines()
    .map(|s| s.to_string())
    .filter(|s| !s.starts_with("commit "))
    .map(|s| CommitDetails::from_str(&s))
    .rev()
    .collect()
}

fn create_branches(git: &Git, names: &[String], from_branch: &str) {
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

fn push_branches(
    git: &Git,
    commit_details: &[CommitDetails],
    target_branch: &str,
    dependent: bool,
) {
    for (i, commit_detail) in commit_details.iter().enumerate() {
        let subject = if i != 0 && dependent {
            format!("{} {}", "Draft: ", commit_detail.subject)
        } else {
            commit_detail.subject.to_string()
        };
        let push_options = get_default_push_options(target_branch, &subject, "");
        git.push(&commit_detail.branch_name, &push_options);
        println!("Pushed {}", &commit_detail.branch_name);
    }
}

fn rebase_commits_onto_branches(git: &Git, commit_details: &[CommitDetails]) {
    for commit_detail in commit_details.iter() {
        println!("Created branch {}", commit_detail);
        git.rebase(&commit_detail.commit_sha, &commit_detail.branch_name);
    }
}

fn pick_commits_onto_branches(git: &Git, commit_details: &[CommitDetails]) {
    for commit_detail in commit_details.iter() {
        let default_branch = git.get_default_branch();
        git.cherry_pick(
            &default_branch,
            &commit_detail.commit_sha,
            &commit_detail.branch_name,
        );
        println!(
            "Picked commit {} to branch {}",
            &commit_detail.commit_sha, &commit_detail.branch_name
        );
    }
}

fn get_branches(commit_details: &[CommitDetails]) -> Vec<String> {
    commit_details
        .to_owned()
        .into_iter()
        .map(|cd| cd.branch_name)
        .collect()
}

pub fn create_separate_merge_requests(git: &Git, dependent: bool, reset: bool) {
    let default_branch = git.get_default_branch();
    let original_branch = git.get_current_branch();
    let remote = git.get_remote();
    let remote_branch = format!("{}/{}", remote, default_branch);
    let commit_details = get_commit_branch_till_branch(&git, &default_branch, &remote);
    let branches: Vec<String> = get_branches(&commit_details);
    create_branches(&git, &branches, &remote_branch);
    if dependent {
        rebase_commits_onto_branches(&git, &commit_details);
    } else {
        pick_commits_onto_branches(&git, &commit_details);
    }
    push_branches(&git, &commit_details, &default_branch, dependent);
    git.switch(&original_branch);
    if reset {
        git.hard_reset();
    }
}
