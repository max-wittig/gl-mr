use structopt::StructOpt;
mod credentials;
mod git_helper;
use credentials::GitLabCredentials;

#[derive(Debug, StructOpt)]
#[structopt(name = "gl-mr", about = "Usage of gl-mr")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
    let env = GitLabCredentials::from_env();
    println!("{:?}", env);
    git_helper::get_project_id();
}
