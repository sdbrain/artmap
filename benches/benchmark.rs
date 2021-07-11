#[macro_use]
extern crate criterion;
extern crate art_rs;

use art_rs::Art;
use criterion::Criterion;
use criterion::{black_box, BatchSize, Benchmark};
use radix_trie::Trie;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

static PATH: &str = "/usr/share/dict/words";
static PATH_RANDOM_NOS: &str = "data/random_nos.txt";
const SEARCH_LIMIT: usize = 80000;

fn insert_radix_trie() {
    let mut map = Trie::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap();
        map.insert(st.clone(), st.clone());
    }
}

fn insert_art() {
    let mut map = Art::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap().as_bytes().to_vec();
        map.insert(st.clone(), st);
    }
}

fn insert_hash_map() {
    let mut map = HashMap::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap();
        map.insert(st.clone(), st.clone());
    }
}

fn insert_radix_trie_b(c: &mut Criterion) {
    c.bench_function("insert_radix_trie", |b| b.iter(|| insert_radix_trie()));
}

fn insert_art_b(c: &mut Criterion) {
    c.bench_function("insert_art", |b| b.iter(|| insert_art()));
}

fn insert_hash_map_b(c: &mut Criterion) {
    c.bench_function("insert_simple_hashmap", |b| b.iter(|| insert_hash_map()));
}

fn search_simple_trie(map: Rc<Art>) {
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for (index, line) in input.lines().enumerate() {
        if index == SEARCH_LIMIT {
            break;
        }
        let st = line.unwrap();
        let st = st.as_bytes().to_vec();
        map.search(&st);
    }
}

fn search_art_b(c: &mut Criterion) {
    let mut map = Art::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap();
        let st = st.as_bytes().to_vec();
        map.insert(st.clone(), st);
    }
    let map = Rc::new(map);

    c.bench_function("search_art", move |b| {
        b.iter_batched(
            || map.clone(),
            |map| search_simple_trie(map),
            BatchSize::LargeInput,
        )
    });
}

fn search_radix_trie(map: Rc<Trie<String, String>>) {
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for (index, line) in input.lines().enumerate() {
        if index == SEARCH_LIMIT {
            break;
        }
        let st = line.unwrap();
        map.get(&st);
    }
}

fn search_radix_trie_b(c: &mut Criterion) {
    let mut map = Trie::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap();
        map.insert(st.clone(), st.clone());
    }
    let map = Rc::new(map);

    c.bench_function("search_radix", move |b| {
        b.iter_batched(
            || map.clone(),
            |map| search_radix_trie(map),
            BatchSize::LargeInput,
        )
    });
}

fn search_hash_map(map: Rc<HashMap<String, String>>) {
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for (index, line) in input.lines().enumerate() {
        if index == SEARCH_LIMIT {
            break;
        }
        let st = line.unwrap();
        map.get(&st);
    }
}

fn search_hash_map_b(c: &mut Criterion) {
    let mut map = HashMap::new();
    let input = File::open(PATH).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let st = line.unwrap();
        map.insert(st.clone(), st.clone());
    }
    let map = Rc::new(map);
    c.bench_function("search_hashmap", move |b| {
        b.iter_batched(
            || map.clone(),
            |map| search_hash_map(map),
            BatchSize::LargeInput,
        )
    });
}

fn search_integer_simple_trie(map: Rc<Art>) {
    let input = File::open(PATH_RANDOM_NOS).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let val= line.unwrap();
        map.search(&val.as_bytes());
    }
}

fn search_art_integers_b(c: &mut Criterion) {
    let mut map = Art::new();
    for i in 0..1000000 {
        let st = format!("{}", i).as_bytes().to_vec();
        map.insert(st.clone(), st);
    }
    let map = Rc::new(map);
    c.bench_function("search_integer_art", move |b| {
        b.iter_batched(
            || map.clone(),
            |map| search_integer_simple_trie(map),
            BatchSize::LargeInput,
        )
    });
}

fn search_integer_hash_map(map: Rc<BTreeMap<i32, i32>>) {
    let input = File::open(PATH_RANDOM_NOS).unwrap();
    let input = BufReader::new(input);
    for line in input.lines() {
        let val: i32 = line.unwrap().parse().unwrap();
        map.get(&val);
    }
}

fn search_hash_map_integers_b(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    for i in 0..1000000 {
        map.insert(i, i);
    }
    let map = Rc::new(map);
    c.bench_function("search_integer_hashmap", move |b| {
        b.iter_batched(
            || map.clone(),
            |map| search_integer_hash_map(map),
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(
    benches,
   insert_art_b,
   insert_radix_trie_b,
   insert_hash_map_b,
    search_art_b,
    search_radix_trie_b,
    search_hash_map_b,
     search_hash_map_integers_b,
     search_art_integers_b
);
criterion_main!(benches);
