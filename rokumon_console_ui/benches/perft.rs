use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rokumon_core::board::Layout;
use rokumon_core::card::Deck;
use rokumon_core::game::{Game, Rules};
use rokumon_console_ui::perft::perft;

pub fn start_pos(c: &mut Criterion) {
    let deck = Deck::ordered("jggjgjj").unwrap();

    let mut group = c.benchmark_group("perft @start_pos !f !s");
    let mut game = Game::new(Layout::Bricks7, deck.clone(), Rules::new(false, false));
    group.bench_function("d=3", |b| b.iter(|| perft(&mut game, black_box(3))));
    let mut game = Game::new(Layout::Bricks7, deck.clone(), Rules::new(false, false));
    group.bench_function("d=4", |b| b.iter(|| perft(&mut game, black_box(4))));
    group.finish();

    let mut group = c.benchmark_group("perft @start_pos f !s");
    let mut game = Game::new(Layout::Bricks7, deck.clone(), Rules::new(true, false));
    group.bench_function("d=3", |b| b.iter(|| perft(&mut game, black_box(3))));
    let mut game = Game::new(Layout::Bricks7, deck.clone(), Rules::new(false, false));
    group.bench_function("d=4", |b| b.iter(|| perft(&mut game, black_box(4))));
    group.finish();

    let mut group = c.benchmark_group("perft @start_pos f s");
    let mut game = Game::new(Layout::Bricks7, deck.clone(), Rules::new(true, true));
    group.bench_function("d=2", |b| b.iter(|| perft(&mut game, black_box(2))));
    group.finish();
}

criterion_group!(benches, start_pos);
criterion_main!(benches);
