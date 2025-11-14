fn main() {
    glazer::run(
        acid::Acid::default(),
        glazer::static_frame_buffer!(800, 800, u32, 0),
        800,
        800,
        |_| {},
        acid::update_and_render,
        Some("target/debug/libacid.so"),
    );
}
