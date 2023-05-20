#![feature(binary_heap_drain_sorted)]

use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    env, fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rayon::prelude::*;

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The files and directories to analyse
    entries: Vec<PathBuf>,

    /// Whether to print GNU-style human readable format (e.g 10M for 10 megabytes)
    #[arg(short = 'H', long = "human")]
    human: bool,
}

fn main() {
    let args = Args::parse();

    let heap = Mutex::new(BinaryHeap::new());

    args.entries.into_par_iter().for_each(|path| {
        let size = get_size(&path);
        heap.lock().unwrap().push(SizeOrErr(size, path));
    });

    heap.lock()
        .unwrap()
        .drain_sorted()
        .map(|s| (s.0, s.1))
        .for_each(|(size, name)| {
            let name = name.to_string_lossy();
            match size {
                Ok(size) => println!("{} {}", human_size(size), name),
                Err(err) => eprintln!("Errored attempting '{}', err = {}", name, err),
            }
        });
}

struct SizeOrErr(io::Result<u64>, PathBuf);

impl PartialEq for SizeOrErr {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for SizeOrErr {}

impl PartialOrd for SizeOrErr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SizeOrErr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0 {
            Ok(len) => other.0.as_ref().map_or(Ordering::Equal, |o| len.cmp(o)),
            Err(_) => match other.0 {
                Ok(_) => Ordering::Less,
                Err(_) => Ordering::Equal,
            },
        }
    }
}

fn human_size(bytes: u64) -> String {
    let magnitude = if bytes == 0 {
        0
    } else {
        (64 - 1 - bytes.leading_zeros()) / 10
    };

    let suffix = match magnitude {
        0 => "B",
        1 => "K",
        2 => "M",
        3 => "G",
        4 => "T",
        5 => "P",
        _ => "<ginormous!>",
    };

    let adjusted = bytes as f64 / 1024f64.powi(magnitude.try_into().unwrap());

    format!(
        "{:>3.N$}{}",
        adjusted,
        suffix,
        N = if adjusted > 10.0 { 0 } else { 1 },
    )
}

fn get_size(path: &Path) -> io::Result<u64> {
    if path.is_dir() {
        fs::read_dir(path)?
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|entry| -> io::Result<u64> {
                let entry = entry?;
                get_size(&entry.path())
            })
            .sum()
    } else {
        Ok(path.metadata()?.len())
    }
}
