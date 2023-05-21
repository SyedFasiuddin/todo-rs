use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use std::time::SystemTime;

#[derive(Hash, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

#[derive(Serialize, Deserialize, Default)]
struct State {
    todos: Vec<String>,
    mon: Vec<char>,
    tue: Vec<char>,
    wed: Vec<char>,
    thu: Vec<char>,
    fri: Vec<char>,
    sat: Vec<char>,
    sun: Vec<char>,
}

impl State {
    fn new() -> Self {
        Default::default()
    }

    fn read_state_from_file(file_loc: &str) -> Self {
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

        serde_yaml::from_str(&buf).expect("Failed to deserialize from string")
    }

    fn add_todo(&mut self, todo: &str) {
        self.todos.push(todo.to_string());
    }

    fn save_completion_info(&mut self, today: Day, v: Vec<char>) {
        match today {
            Day::Mon => self.mon = v,
            Day::Tue => self.tue = v,
            Day::Wed => self.wed = v,
            Day::Thu => self.thu = v,
            Day::Fri => self.fri = v,
            Day::Sat => self.sat = v,
            Day::Sun => self.sun = v,
        }
    }

    fn save_to_file(&self, file_loc: &str) {
        let content = serde_yaml::to_string(self).expect("Failed to serialize for writing to file");
        let mut fd = match File::create(file_loc) {
            Ok(fd) => fd,
            Err(e) => {
                eprintln!("Failed creating file: {file_loc} due to: {e}");
                exit(1);
            }
        };

        match writeln!(fd, "{content}") {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed writing serialized data to file due to: {e}");
                exit(1);
            }
        };
    }
}

fn get_day(day_of_week: u64) -> Day {
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

    get_day(day_of_week)
}

fn get_last_accessed_day(file_loc: &str) -> Day {
    let last_accessed_time = File::open(file_loc)
        .expect("Something went wrong with opening the file")
        .metadata()
        .expect("Something went wrong with getting the metadata of file")
        .modified()
        .expect("Sometihng went wrong with getting the last modified time of file");
    let duration = last_accessed_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Something went wrong while calculating the duration since epoch");

    get_day((duration.as_secs() / (24 * 60 * 60)) % 7)
}

fn get_file_location() -> String {
    match env::var("XDG_DATA_HOME") {
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

fn get_completion_info(todos: &[String]) -> Vec<char> {
    let mut buf: Vec<char> = Vec::new();

    println!("Tick off the things done today:");
    for item in todos {
        print!("{item} (Y/n) ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).expect("Unable to get input");

        if input.len() > "?\n".len() {
            buf.push('X');
            continue;
        }

        input.pop(); // new line
        match input.pop() {
            Some('Y' | 'y') => buf.push('Y'),
            Some(_) => buf.push('X'),
            None => buf.push('Y'), // Default case, if directly pressed enter
        }
    }
    buf
}

fn main() {
    let today = get_todays_day();
    let file_loc = get_file_location();

    if today == get_last_accessed_day(&file_loc) {
        println!("Already entered completion info for today, see you tomorrow.");
        exit(1);
    }

    match today {
        Day::Mon => {
            print!(
                "What are you planning to do everyday of this week?\n\
                 Enter a comma separated list of items: "
            );
            stdout().flush().unwrap();

            let mut input = String::new();
            stdin().read_line(&mut input).expect("Unable to get input");

            let mut state = State::new();
            for todo in input.split(',') {
                state.add_todo(todo);
            }

            let input = get_completion_info(&state.todos[..]);
            state.save_completion_info(today, input);
            state.save_to_file(&file_loc);
        }

        Day::Sun => {
            let mut state = State::read_state_from_file(&file_loc);
            let input = get_completion_info(&state.todos[..]);
            state.save_completion_info(today, input);

            let mut max_length = usize::MIN;
            for todo in &state.todos {
                max_length = max(todo.len(), max_length);
            }

            let mut lines: Vec<String> = vec![];
            for (idx, todo) in state.todos.iter().enumerate() {
                // todo item name
                let mut str = format!("┃ {: <max_length$}", todo);

                str.push_str(&format!(" ┃  {} ", state.mon.get(idx).unwrap_or(&'?')));
                str.push_str(&format!(" ┃  {} ", state.tue.get(idx).unwrap_or(&'?')));
                str.push_str(&format!(" ┃  {} ", state.wed.get(idx).unwrap_or(&'?')));
                str.push_str(&format!(" ┃  {} ", state.thu.get(idx).unwrap_or(&'?')));
                str.push_str(&format!(" ┃  {} ", state.fri.get(idx).unwrap_or(&'?')));
                str.push_str(&format!(" ┃  {} ", state.sat.get(idx).unwrap_or(&'?')));
                str.push_str(&format!(" ┃  {}  ┃", state.sun.get(idx).unwrap_or(&'?')));

                lines.push(str);
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
            for line in lines {
                println!("{line}");
            }
            println!("{foot}");
        }

        _ => {
            let mut state = State::read_state_from_file(&file_loc);
            let input = get_completion_info(&state.todos[..]);
            state.save_completion_info(today, input);
            state.save_to_file(&file_loc);
        }
    }
}
