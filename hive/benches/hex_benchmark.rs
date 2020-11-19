use criterion::{criterion_group, criterion_main, Criterion};
use hive::hex;

fn big_board() -> Vec<hex::Hex> {
    let mut head = hex::ORIGIN;
    let mut board = vec![head];
    let n = 25;
    for _ in 0..n {
        board.push(head.e());
        head = head.e();
    }
    for i in 0..board.len() {
        if i % 2 == 0 {
            board.push(board[i].ne());
            board.push(board[i].ne().ne());
        }
        if i % 2 == 1 {
            board.push(board[i].se());
            board.push(board[i].se().se());
        }
    }
    board
}

pub fn get_empty_neighbors_benchmark(c: &mut Criterion) {
    let board = big_board();
    c.bench_function("get_empty_neighbors", |b| b.iter(|| hex::Hex::get_empty_neighbors(&board)));
}

pub fn all_contiguous_benchmark(c: &mut Criterion) {
    let board = big_board();
    c.bench_function("all_contiguous", |b| b.iter(|| hex::Hex::all_contiguous(&board)));
}

criterion_group!(hex_benches, get_empty_neighbors_benchmark, all_contiguous_benchmark);
criterion_main!(hex_benches);
