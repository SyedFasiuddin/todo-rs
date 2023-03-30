enum Day {
    Mon, Tue, Wed, Thu, Fri, Sat, Sun,
}

fn get_todays_day() -> Day {
    use std::time::SystemTime;

    let day_of_week: u64;

    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            let sec = n.as_secs();
            let days_since_epoch = sec / (24 * 60 * 60);
            day_of_week = days_since_epoch % 7
        }
        Err(e) => {
            eprintln!("Failed getting system time due to: {e}");
            std::process::exit(1);
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
        _ => unreachable!("Wrong day of the week, this should never be the case unless my
                          calculations are wrong"),
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
            std::process::exit(1);
        }
    }
}

fn read_todos_from_file(file_loc: &str) -> Vec<String> {
    use std::fs::File;
    use std::io::Read;

    let mut buf = String::new();

    match File::open(file_loc) {
        Ok(mut fd) => match fd.read_to_string(&mut buf) {
            Ok(_size) => 0,
            Err(e) => {
                eprintln!("Failed to read from file: {file_loc} due to: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to open the file due to: {e}");
            std::process::exit(1);
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
    use std::fs::File;
    use std::io::{Read, Write};

    let mut buf = String::new();

    let mut fd = match File::open(file_loc) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("Failed reading file: {file_loc} due to: {e}");
            std::process::exit(1);
        }
    };
    fd.read_to_string(&mut buf);

    let mut fd = match File::create(file_loc) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("Failed creating file: {file_loc} due to: {e}");
            std::process::exit(1);
        }
    };

    buf.push_str(todays_items);

    for line in buf.lines() {
        writeln!(fd, "{line}");
    }
}

fn main() {
    use std::io::{Read, Write};

    let file_loc = get_file_location();
    let todos = read_todos_from_file(&file_loc);

    let mut buf = String::new();

    println!("Tick off the things done today:");
    for item in todos {
        print!("{item} (Y/n) ");
        std::io::stdout().flush();

        let mut input = String::new();
        std::io::Stdin::read_line(&std::io::stdin(), &mut input);

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

    write_todays_items_to_file(&file_loc, &buf);
}
