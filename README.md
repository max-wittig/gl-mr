# gl-mr

It transforms commits on one branch into merge requests.

Creates Merge-Requests on GitLab based on the commits in a separate branch.

This is quite handy, if you work with feature branches where your goal is to create separate
merge requests. Using the `d` flag, you can create MRs that depend on each other.

![image](/uploads/011436e735e599f42acd7dd254ef8d5d/image.png)

## Usage

```sh
USAGE:
    gl-mr [FLAGS] [OPTIONS]

FLAGS:
    -d, --dependent    Enable dependent commit mode. Useful, when you want to have several MRs that depend on each other
        --dry          Activate dry mode
    -h, --help         Prints help information
    -r, --reset        Reset branch hard to upstream, after push
    -V, --version      Prints version information

OPTIONS:
    -a, --assignee <assignee>    Set assignee
    -g, --git <git>      Git executable. Defaults to the executable that's in $PATH [default: git]
    -p, --path <path>    Path to the git repository. Defaults to the current working directory [default: .]
```
