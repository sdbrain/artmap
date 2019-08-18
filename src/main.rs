use art_rs::Art;
use std::fs::File;
use std::io;
use std::io::{stdin, BufRead, BufReader};

const SEARCH_LIMIT: usize = 1000;
static PATH: &str = "/tmp/words.txt";

pub fn main() {
    let mut map = Art::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap().as_bytes().to_vec();
        map.insert(st.clone(), st);
    }

    println!("Finished reading file");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("could not read");

    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for (index, line) in input.lines().enumerate() {
        if index == SEARCH_LIMIT {
            break;
        }
        let st = line.unwrap();
        let st = st.as_bytes().to_vec();
        let res = map.search(&st);
    }
    println!("Finished searching");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("could not read");
}
