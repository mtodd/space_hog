extern crate rustc_serialize;
extern crate docopt;

use std::fmt;
use std::env;
use std::fs;
use std::path::PathBuf;

use std::sync::mpsc::{channel,Sender,Receiver};
use std::thread;

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

impl fmt::Display for FileWithSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileWithSize {{ path: {}, size: {} }}", self.path.display(), self.size)
    }
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

    let (path_tx, path_rx) = channel::<(PathBuf, fs::Metadata)>();
    let (out_tx,  out_rx)  = channel::<FileWithSize>();

    // path scanner
    // thread::spawn(move|| {
    //     match path_rx.recv() {
    //         Err(why) => { panic!("dir") },
    //         Ok((path, metadata)) => {
    //             if !metadata.is_dir() {
    //                 file_tx.send((path, metadata));
    //             }
    //         }
    //     };
    // });
    // thread::spawn(move|| {
    //     match file_rx.recv() {
    //         Err(why) => { panic!("file {}", why) },
    //         Ok((path, metadata)) => {
    //             out_tx.send(FileWithSize { path: path.clone(), size: metadata.len() });
    //         }
    //     };
    // });

    // output
    thread::spawn(move|| {
        loop {
            match out_rx.recv() {
                Err(why) => {},
                Ok(file) => {
                    println!("{}\t{}", file.path.display(), file.size);
                }
            };
        };
    });

    // println!("{}", out_rx.recv().unwrap());

    scan_dir(path, 0, &out_tx);
}

// scan through directories
fn scan_dir(path: PathBuf, mut currdepth: u64, out_tx: &Sender<FileWithSize>) {
    match fs::read_dir(&path) {
        Err(why) => {},
        Ok(paths) => {
            for path in paths {
                match path {
                    Err(why) => { panic!("path {}", why) },
                    Ok(path) => {
                        match fs::metadata(&path.path().clone()) {
                            Err(why) => { panic!("metadata for {}", path.path().display()) },
                            Ok(metadata) => {
                                if metadata.is_dir() {
                                    scan_dir(path.path(), currdepth + 1, &out_tx);
                                } else {
                                    out_tx.send(FileWithSize { path: path.path().clone(), size: metadata.len() });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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
