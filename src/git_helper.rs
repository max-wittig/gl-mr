use git2::Repository;


fn get_remote_url() {
    let repo = Repository::open(".").unwrap();
    for remote_name in repo.remotes().unwrap().iter() {
        let remote = repo.find_remote(remote_name.unwrap());
        println!("{:?}", remote.unwrap().url());
    }
}

pub fn get_project_id() -> u64 {
    get_remote_url();
    42
}
