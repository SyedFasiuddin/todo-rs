use std::fs::File;
use std::io::{stdin, stdout, Read, Stdin, Write};
use std::process::exit;
use std::time::SystemTime;

enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

fn get_todays_day() -> Day {
    let day_of_week = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            let sec = n.as_secs();
            let days_since_epoch = sec / (24 * 60 * 60);
            days_since_epoch % 7
        }
        Err(e) => {
            eprintln!("Failed getting system time due to: {e}");
            exit(1);
        }
    };

    // UNIX_EPOCH i.e. Jan 1, 1970 was Thu which means 0 will be Thu
    match day_of_week {
        0 => Day::Thu,
        1 => Day::Fri,
        2 => Day::Sat,
        3 => Day::Sun,
        4 => Day::Mon,
        5 => Day::Tue,
        6 => Day::Wed,
        _ => unreachable!(
            "Wrong day of the week, this should never be the case unless my calculations are wrong"
        ),
    }
}

fn get_file_location() -> String {
    match std::env::var("XDG_DATA_HOME") {
        Ok(mut val) => {
            val.push_str("/TODO");
            val
        }
        Err(e) => {
            eprintln!("Failed reading envionment variable $XDG_DATA_HOME due to {e}");
            exit(1);
        }
    }
}

fn read_todos_from_file(file_loc: &str) -> Vec<String> {
    let mut buf = String::new();

    match File::open(file_loc) {
        Ok(mut fd) => match fd.read_to_string(&mut buf) {
            Ok(_size) => 0,
            Err(e) => {
                eprintln!("Failed to read from file: {file_loc} due to: {e}");
                exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to open the file due to: {e}");
            exit(1);
        }
    };

    let first_line = buf.lines().next().unwrap();

    let mut todos: Vec<String> = vec![];
    for str in first_line.split_whitespace() {
        todos.push(str.to_string());
    }

    todos
}

fn write_todays_items_to_file(file_loc: &str, todays_items: &str) {
    let mut buf = String::new();

    let mut fd = match File::open(file_loc) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("Failed reading file: {file_loc} due to: {e}");
            exit(1);
        }
    };
    match fd.read_to_string(&mut buf) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed reading file {file_loc} due to: {e}\nAborting due to previous error");
            exit(1);
        }
    };

    let mut fd = match File::create(file_loc) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("Failed creating file: {file_loc} due to: {e}");
            exit(1);
        }
    };

    buf.push_str(todays_items);

    for line in buf.lines() {
        match writeln!(fd, "{line}") {
            Ok(()) => continue,
            Err(e) => {
                eprintln!(
                    "Failed writing to file: {fd:?} due to: {e}\nAborting due to previous error"
                );
                exit(1);
            }
        }
    }
}

fn get_completion_info(todos: Vec<String>) -> String {
    let mut buf = String::new();

    println!("Tick off the things done today:");
    for item in todos {
        print!("{item} (Y/n) ");
        stdout().flush().unwrap();

        let mut input = String::new();
        Stdin::read_line(&stdin(), &mut input).expect("Unable to get input");

        if input.len() > "?\n".len() {
            buf.push_str("X ");
            continue;
        }

        input.pop(); // new line
        match input.pop() {
            Some('Y' | 'y') => buf.push_str("Y "),
            Some(_x) => buf.push_str("X "),
            None => buf.push_str("Y "), // Default case, if directly pressed enter
        }
    }
    buf
}

fn main() {
    let today = get_todays_day();
    let file_loc = get_file_location();

    match today {
        Day::Mon => {
            let mut fd = match File::create(&file_loc) {
                Ok(fd) => fd,
                Err(e) => {
                    eprintln!("Failed reading file: {file_loc} due to: {e}");
                    exit(1);
                }
            };

            println!("What are you planning to do everyday of this week?");
            print!("Enter a comma separated list of items: ");
            stdout().flush().unwrap();

            let mut input = String::new();
            Stdin::read_line(&stdin(), &mut input).expect("Unable to get input");

            let mut todos: Vec<String> = vec![];
            for todo in input.split(',') {
                todos.push(todo.trim().to_string());
            }

            for item in &todos {
                write!(fd, "{item} ").expect("unable to write to file");
            }
            writeln!(fd).expect("unable to write to file");

            let input = get_completion_info(todos);
            write_todays_items_to_file(&file_loc, &input);
        }

        Day::Sun => {
            let todos = read_todos_from_file(&file_loc);
            let input = get_completion_info(todos);
            let input: Vec<&str> = input.split_whitespace().collect();

            let mut buf = String::new();

            match File::open(&file_loc) {
                Ok(mut fd) => match fd.read_to_string(&mut buf) {
                    Ok(_size) => 0,
                    Err(e) => {
                        eprintln!("Failed to read from file: {file_loc} due to: {e}");
                        exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to open the file due to: {e}");
                    exit(1);
                }
            };

            let mut todos: Vec<String> = vec![];

            let lines = buf.lines().collect::<Vec<&str>>();
            let mut max_length = usize::MIN;

            let first_line = lines.clone();
            let first_line = first_line.first().unwrap();
            let t = first_line.split_whitespace().collect::<Vec<&str>>();
            for todo in t {
                max_length = std::cmp::max(todo.len(), max_length);
            }

            for (idx, line) in lines.iter().enumerate() {
                let x = line.split_whitespace().collect::<Vec<&str>>();
                if idx == 0 {
                    for y in x.iter() {
                        todos.push(format!("┃ {: <max_length$}", y));
                    }
                    continue;
                }
                for (idx, done) in x.iter().enumerate() {
                    todos[idx].push_str(" ┃  ");
                    todos[idx].push_str(done);
                    todos[idx].push(' ');
                }
            }
            for (idx, todo) in todos.iter_mut().enumerate() {
                todo.push_str(" ┃  ");
                todo.push_str(input[idx]);
                todo.push_str("  ┃");
            }

            let head = format!(
                "Weekly Stuff:\n\
                ┏━{:━>max_length$}━┳━━━━━┳━━━━━┳━━━━━┳━━━━━┳━━━━━┳━━━━━┳━━━━━┓\n\
                ┃ {: >max_length$} ┃ Mon ┃ Tue ┃ Wed ┃ Thu ┃ Fri ┃ Sat ┃ Sun ┃\n\
                ┣━{:━>max_length$}━╋━━━━━╋━━━━━╋━━━━━╋━━━━━╋━━━━━╋━━━━━╋━━━━━┫",
                "━", " ", "━"
            );

            let foot = format!(
                "┗━{:━>max_length$}━┻━━━━━┻━━━━━┻━━━━━┻━━━━━┻━━━━━┻━━━━━┻━━━━━┛",
                "━"
            );

            println!("{head}");
            for todo in todos {
                println!("{todo}");
            }
            println!("{foot}");
        }

        _ => {
            let todos = read_todos_from_file(&file_loc);
            let input = get_completion_info(todos);
            write_todays_items_to_file(&file_loc, &input);
        }
    }
}
