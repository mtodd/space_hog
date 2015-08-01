extern crate rustc_serialize;
extern crate docopt;

use std::env;
use std::fs;
use std::path::PathBuf;

// https://github.com/docopt/docopt.rs
use docopt::Docopt;

static USAGE: &'static str = "
Space Hog: find large files recursively.

Usage:
  space_hog [<path>]
  space_hog [<path>] [--recursive] [--threshold=<bytes>] [--depth=<depth>]
  space_hog (-h | --help)
  space_hog --version

Options:
  -h --help             Show this screen.
  --version             Show version.
  --recursive           Recursively scan.
  --threshold=<bytes>   Threshold, in bytes.
  --depth=<depth>       Depth limit.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_recursive: bool,
    flag_threshold: Option<u64>,
    flag_depth: Option<u64>,
    arg_path: Option<String>,
}

struct FileWithSize {
    path: PathBuf,
    size: u64
}

const DEFAULT_THRESHOLD: u64 = 150; // bytes

fn main() {
    let mut args: Args = Docopt::new(USAGE)
                                .and_then(|d| d.decode())
                                .unwrap_or_else(|e| e.exit());
    // println!("{:?}", args);

    let path = match args.arg_path {
        None => env::current_dir().unwrap(),
        Some(ref path) => PathBuf::from(path),
    };

    // println!("The current directory is {}", path.display());

    let mut files: Vec<FileWithSize> = Vec::new();

    scan(&mut files, path, 0, &args);
}

fn scan(files: &mut Vec<FileWithSize>, path: PathBuf, mut currdepth: u64, args: &Args) {
    let threshold = args.flag_threshold.unwrap_or_else(|| DEFAULT_THRESHOLD);

    match fs::read_dir(&path) {
        Err(why) => {},
        Ok(paths) => {
            currdepth += 1;

            for path in paths {
                let upath = path.unwrap().path();

                match fs::metadata(&upath) {
                    Err(why) => {},
                    Ok(metadata) => {
                        let file_size = metadata.len();

                        // report this file
                        if !metadata.is_dir() && file_size > threshold {
                            println!("{}\t{}", upath.display(), file_size);
                            files.push(FileWithSize { path: upath.clone(), size: file_size });
                        }

                        // recurse directories
                        if args.flag_recursive && (args.flag_depth == None || currdepth < args.flag_depth.unwrap()) && metadata.is_dir() {
                            scan(files, upath.clone(), currdepth, args);
                        }
                    }
                }
            }
        }
    }
}
