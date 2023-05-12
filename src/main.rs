#![allow(clippy::needless_range_loop)]

use std::{env, error::Error, fs, process};

use ansi_term::Color;
use lazy_static::lazy_static;

lazy_static! {
    static ref TRACE: bool = env::var("TRACE").map(|val| val == "1").unwrap_or(false);
}

#[derive(Clone, Debug)]
enum Action {
    Add(usize, String),
    Remove(usize, String),
    Substitute(usize, String, String),
    Ignore(usize, String),
}

fn dump(actions: &[Vec<Option<(usize, Action)>>]) {
    if *TRACE {
        assert!(!actions.is_empty());
        for row in actions.iter() {
            for val in row.iter() {
                if let Some((x, a)) = val {
                    let action = match a {
                        Action::Add(_, _) => "A",
                        Action::Remove(_, _) => "R",
                        Action::Substitute(_, _, _) => "S",
                        Action::Ignore(_, _) => "I",
                    };
                    print!("{x:>3} ({action}) ");
                } else {
                    print!("{:>7} ", "-");
                }
            }
            println!();
        }
        println!();
    }
}

fn backtrack_actions(actions: &[Vec<Option<(usize, Action)>>]) -> Vec<&Action> {
    let mut result = vec![];
    let mut n1 = actions.len();
    let mut n2 = actions.get(0).map(|arr| arr.len()).unwrap_or(0);
    while n1 > 0 && n2 > 0 {
        let action = &actions[n1 - 1][n2 - 1].as_ref().unwrap().1;
        result.push(action);
        match action {
            Action::Add(_, _) => n2 -= 1,
            Action::Remove(_, _) => n1 -= 1,
            Action::Substitute(_, _, _) | Action::Ignore(_, _) => {
                n1 -= 1;
                n2 -= 1;
            }
        }
    }
    result
}

fn lev<T: Eq + ToString>(s1: &[T], s2: &[T]) -> Vec<Action> {
    let n1 = s1.len();
    let n2 = s2.len();
    let mut actions = Vec::with_capacity(n1 + 1);
    for _ in 0..n1 + 1 {
        actions.push(vec![None; n2 + 1]);
    }

    actions[0][0] = Some((0, Action::Ignore(0, String::from(""))));

    for n2 in 1..n2 + 1 {
        let n1 = 0;
        actions[n1][n2] = Some((n2, Action::Add(n2, s2[n2 - 1].to_string())));
        dump(&actions);
    }
    for n1 in 1..n1 + 1 {
        let n2 = 0;
        actions[n1][n2] = Some((n1, Action::Remove(n1, s1[n1 - 1].to_string())));
        dump(&actions);
    }
    for n1 in 1..n1 + 1 {
        for n2 in 1..n2 + 1 {
            if s1[n1 - 1] == s2[n2 - 1] {
                let x = actions[n1 - 1][n2 - 1].as_ref().map(|tup| tup.0).unwrap();
                actions[n1][n2] = Some((x, Action::Ignore(n2, s1[n1 - 1].to_string())));
                dump(&actions);
                continue;
            }
            actions[n1][n2] = vec![
                (
                    1 + actions[n1 - 1][n2].as_ref().map(|tup| tup.0).unwrap(),
                    Action::Remove(n1, s1[n1 - 1].to_string()),
                ),
                (
                    1 + actions[n1][n2 - 1].as_ref().map(|tup| tup.0).unwrap(),
                    Action::Add(n2, s2[n2 - 1].to_string()),
                ),
                (
                    1 + actions[n1 - 1][n2 - 1].as_ref().map(|tup| tup.0).unwrap(),
                    Action::Substitute(n2, s1[n1 - 1].to_string(), s2[n2 - 1].to_string()),
                ),
            ]
            .iter()
            .min_by_key(|item| item.0)
            .cloned();
            dump(&actions);
        }
    }
    backtrack_actions(&actions)
        .into_iter()
        .cloned()
        .rev()
        .skip(1)
        .collect()
}

fn print_actions(actions: &[Action]) {
    let width = f32::log10(actions.len() as f32) as usize + 1;
    let yellow = Color::Yellow;
    let red = Color::Red;
    let green = Color::Green;
    for action in actions {
        match action {
            Action::Add(row, line) => {
                println!(
                    "{row:>width$} {action}| {line}",
                    action = green.paint("+"),
                    line = green.paint(line),
                );
            }
            Action::Remove(row, line) => {
                println!(
                    "{row:>width$} {action}| {line}",
                    action = red.paint("-"),
                    line = red.paint(line),
                );
            }
            Action::Substitute(row, line1, line2) => {
                println!(
                    "{row:>width$} {action}| {line1} ⇆  {line2}",
                    action = yellow.paint("~"),
                    line1 = red.paint(line1),
                    line2 = green.paint(line2),
                );
            }
            Action::Ignore(row, line) => {
                println!("{row:>width$}  | {line}");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("Not enough arguments");
        process::exit(1);
    }
    let s1 = fs::read_to_string(&args[1])?;
    let s2 = fs::read_to_string(&args[2])?;
    let actions = lev(
        &s1.lines().collect::<Vec<&str>>(),
        &s2.lines().collect::<Vec<_>>(),
    );
    print_actions(&actions);
    Ok(())
}
