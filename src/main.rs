use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process;

static USAGE: &'static str = "
Space Hog: find large files recursively.

Usage:
  space_hog <path>
  space_hog <path> [-r] [-t=<bytes>] [-d=<depth>]
  space_hog (-h | --help)
  space_hog --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  -r            Recursively scan.
  -t=<bytes>    Threshold, in bytes.
  -d=<depth>    Depth limit.
";

struct Args {
    flag_r: bool,
    flag_t: Option<u64>,
    flag_d: Option<u64>,
    arg_path: PathBuf,
}

struct FileWithSize {
    path: PathBuf,
    size: u64
}

fn main() {
    println!("space_hog [-r] [-t 128] .");
    println!("recursively find large files");
    println!("");

    let pwd = env::current_dir().unwrap();
    let args: Args = Args { flag_r: true, flag_t: Some(150), flag_d: None, arg_path: pwd.clone() };

    let mut currdepth: u64 = 0;

    println!("The current directory is {}", pwd.display());

    let mut files: Vec<FileWithSize> = Vec::new();

    scan(&mut files, pwd.clone(), currdepth, &args);
}

fn scan(files: &mut Vec<FileWithSize>, path: PathBuf, mut currdepth: u64, args: &Args) {
    let paths = fs::read_dir(&path).unwrap();

    currdepth += 1;

    for path in paths {
        let upath = path.unwrap().path();

        let metadata = fs::metadata(&upath).unwrap();
        let file_size = metadata.len();

        // report this file
        if !metadata.is_dir() && file_size > args.flag_t.unwrap() {
            println!("{}\t{}", upath.display(), file_size);
            files.push(FileWithSize { path: upath.clone(), size: file_size });
        }

        // recurse directories
        if args.flag_r && (args.flag_d == None || currdepth < args.flag_d.unwrap()) && metadata.is_dir() {
            scan(files, upath.clone(), currdepth, args);
        }
    }
}
