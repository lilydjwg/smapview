use std::fs::{File,read_dir};
use std::io::Read;
use std::sync::mpsc::channel;

use rayon::prelude::*;
use clap::Parser;

const UNITS: [char; 4] = ['K', 'M', 'G', 'T'];

fn filesize(size: isize) -> String {
  let mut left = size.abs() as f64;
  let mut unit = -1;

  while left > 1100. && unit < 3 {
    left /= 1024.;
    unit += 1;
  }
  if unit == -1 {
    format!("{}B", size)
  } else {
    if size < 0 {
      left = -left;
    }
    format!("{:.1}{}iB", left, UNITS[unit as usize])
  }
}

fn chop_null(mut s: String) -> String {
  let last = s.len() - 1;
  if !s.is_empty() && s.as_bytes()[last] == 0 {
    s.truncate(last);
  }
  s.replace("\0", " ")
}

fn get_comm_for(pid: usize) -> String {
  let cmdline_path = format!("/proc/{}/cmdline", pid);
  let mut buf = String::new();
  let mut file = match File::open(&cmdline_path) {
    Ok(f) => f,
    Err(_) => return String::new(),
  };
  match file.read_to_string(&mut buf) {
    Ok(_) => (),
    Err(_) => return String::new(),
  };
  chop_null(buf)
}

fn get_usage_for(pid: usize, field: &[u8]) -> isize {
  let smaps_path = format!("/proc/{}/smaps_rollup", pid);

  let mut file = match File::open(&smaps_path) {
    Ok(f) => f,
    Err(_) => return 0,
  };

  let mut vec = vec![];
  if file.read_to_end(&mut vec).is_err() {
    return 0
  }
  for line in vec.split(|&c| c == b'\n') {
    if line.starts_with(field) {
      let string = line[field.len()..]
        .iter()
        .skip_while(|&&c| c == b' ')
        .take_while(|&&c| c != b' ')
        .map(|&c| c as char)
        .collect::<String>();
      return string.parse::<isize>().unwrap() * 1024;
    }
  }
  0
}

fn get_usage(field: &[u8]) -> Vec<(usize, isize, String)> {
  rayon::in_place_scope(|pool| {
    let (tx, rx) = channel();
    for d in read_dir("/proc").unwrap() {
      let tx = tx.clone();
      pool.spawn(move |_| {
        let path = d.unwrap().path();
        if let Ok(pid) = path.file_name().unwrap().to_str().unwrap().parse() {
          tx.send(match get_usage_for(pid, field) {
            0 => None,
            usage => Some((pid, usage, get_comm_for(pid))),
          }).unwrap();
        } else {
          tx.send(None).unwrap();
        }
      });
    }
    drop(tx);
    rx.iter().flatten().collect()
  })
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
  /// which /proc/PID/smaps_rollup field to look at
  #[arg(short, long, default_value="Swap")]
  field: String,
}

#[allow(clippy::print_literal)]
fn main() {
  let args = Args::parse();

  let mut field = args.field;
  if !field.ends_with(":") {
    field.push(':');
  }

  // let format = "{:>5} {:>9} {}";
  // let totalFmt = "Total: {:8}";
  let mut usageinfo = get_usage(field.as_bytes());
  usageinfo.par_sort_unstable_by_key(|&(_, size, _)| size);

  let field_name = field.trim_matches(':').to_ascii_uppercase();
  let field_width = field_name.len().max(9);
  println!("{:>7} {:>w$} {}", "PID", field_name, "COMMAND", w = field_width);
  let mut total = 0;
  for &(pid, usage, ref comm) in &usageinfo {
    total += usage;
    println!("{:>7} {:>w$} {}", pid, filesize(usage), comm, w = field_width);
  }
  println!("Total: {:>w$}", filesize(total), w = field_width + 1);
}

// vim: se sw=2:
