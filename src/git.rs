use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

pub struct Git {
    executable: String,
    working_dir: PathBuf,
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

    pub fn get_commits_till_branch(&self, branch_name: String) {
        let result = self.execute(vec!["rev-parse", &branch_name]).expect("Could not get commits!");
        println!("{}", result);
    }

    fn get_default_branch(&self) -> Option<String> {
        Some(String::from("master"))
    }

    fn get_remote(&self) -> Option<String> {
        // TODO: use "origin" by default and ask user for other source, if required
        // This choice should then be remembered and saved somewhere
        Some(String::from("origin"))
    }

    fn push(&self) {}
}

pub fn create_separate_merge_requests(git: &Git) {
    let default_branch = git.get_default_branch().unwrap();
    git.get_commits_till_branch(default_branch);
    // get default branch
    // get commits that differ from main on current branch
    // make separate branches -> name from commit message
    // push separate branches with pushoptions to create MR
}
