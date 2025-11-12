fn main() {
    glazer::run(
        acid::Acid::default(),
        &mut [],
        0,
        0,
        |_| {},
        acid::update_and_render,
        Some("target/debug/libacid.dylib"),
    );
}
