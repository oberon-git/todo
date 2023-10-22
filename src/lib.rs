use std::{env, process, fs, fs::{File, OpenOptions}, io::Write, path::Path};

pub fn run() {
    let args = Args::new();

    match args.op {
        Op::Add(task) => add(task, args.path, args.line_number),
        Op::Remove => remove(args.path, args.line_number),
        Op::List => list(args.path),
    };
}

fn add(task: String, path: String, line_number: i32) {
    let count = count_lines(&path);

    if line_number >= 0 && (line_number as u32) > count + 1 {
        eprintln!("not a valid line number");
        process::exit(1)
    }

    match line_number {
        -1 => {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();

            writeln!(file, "{}. {}", count + 1, task).unwrap();
        },
        ln => {
            let ln = ln as usize;
            
            let mut lines = get_lines(&path);

            let line = format!("{}. {}", ln, task);
            lines.insert((ln - 1) as usize, line);
            for i in ln..lines.len() {
                lines[i] = edit_line_number(&lines[i][..], |ln| ln + 1);
            }
           
            fs::write(&path, lines.join("\n") + "\n").unwrap();
        },
    };
}

fn remove(path: String, line_number: i32) {
    let count = count_lines(&path);
    
    if count == 0 {
        fs::remove_file(&path).unwrap();
        process::exit(1)
    } else if line_number >= 0 && (line_number as u32) > count + 1 {
        eprintln!("not a valid line number");
        process::exit(1)
    }
    
    match line_number {
        -1 => {
            let mut lines = get_lines(&path);
            lines.remove(lines.len() - 1);

            fs::write(&path, lines.join("\n") + "\n").unwrap();
        },
        ln => {
            let ln = ln as usize;

            let mut lines = get_lines(&path);
            lines.remove(ln - 1);
 
            for i in (ln - 1)..lines.len() {
                lines[i] = edit_line_number(&lines[i][..], |ln| ln - 1);
            }

            fs::write(&path, lines.join("\n") + "\n").unwrap();
        },
    };

    if count == 1 {
        fs::remove_file(&path).unwrap();
    }
}

fn list(path: String) {
    println!("{}", fs::read_to_string(&path).unwrap());
}

fn get_lines(path: &str) -> Vec<String> {
    let content = fs::read_to_string(path).unwrap();
    content.lines().map(|l| String::from(l)).collect()
}

fn edit_line_number(line: &str, f: impl FnOnce(usize) -> usize) -> String {
    let mut line = line.split('.');

    let ln = line
        .next()
        .expect("file is formatted incorrectly")
        .parse::<usize>()
        .expect("file is formatted incorrectly");
    
    let ln = (f(ln)).to_string();
    
    let mut line: Vec<&str> = line.collect();
    line.insert(0, &ln[..]);
    line.join(".")
}

fn count_lines(path: &str) -> u32 { 
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .fold(0, |a, _| a + 1)
}

enum Op {
    Add(String),
    Remove,
    List,
}

struct Args {
    op: Op,
    path: String,
    line_number: i32,
}

impl Args {
    fn new() -> Self {
        let usage = "usage: add task | remove | list [-line_num] [-p path]";
        let mut args = env::args();
        
        args.next();

        let op = match args.next() {
            Some(op) => {
                match &op[..] {
                    "add" => {
                        match args.next() {
                            Some(arg) => Op::Add(String::from(arg)),
                            None => {
                                eprintln!("{usage}");
                                process::exit(1)
                            },
                        }
                    },
                    "remove" => Op::Remove,
                    "list" => Op::List,
                    _ => {
                        eprintln!("{usage}");
                        process::exit(1)
                    },
                }
            },
            None => {
                eprintln!("{usage}");
                process::exit(1)
            },
        };

        let args: Vec<String> = args.collect();
        
        let mut path = String::from("./todo.txt");
        let mut line_number = -1;

        // find optinal args
        let _ = args
            .iter()
            .filter(|arg| arg.starts_with('-'))
            .for_each(|arg| {
                let result = &arg[1..].parse::<i32>();
                if let Ok(n) = result {
                    line_number = *n;
                    if line_number <= 0 {
                        eprintln!("line number must be greater than 0");
                        process::exit(1)
                    }
                }
            });

        // find optional args with parameters
        let _ = args
            .windows(2)
            .filter(|w| w[0].starts_with('-'))
            .for_each(|w| {
                let (a, b) = (&w[0], &w[1]);
                if a == "-p"{
                    path = String::from(b);
                }
            });

        if !Path::new(&path).exists() {
            if let Op::Add(_) = op {
                let _ = File::create(&path);
            } else {
                eprintln!("invalid path, todo.txt must exist");
                process::exit(1)
            }
        }

        Self {
            op,
            path,
            line_number,
        }
    }
}
