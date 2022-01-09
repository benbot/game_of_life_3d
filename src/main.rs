fn main() {
    futures::executor::block_on(gameoflife::run(None));
}
