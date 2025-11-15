use acid::Acid;

fn main() {
    glazer::run(
        Acid::default(),
        glazer::static_frame_buffer!(800, 800, u32, 0),
        800,
        800,
        handle_input,
        update_and_render,
        Some("target/debug/libacid.so"),
    );
}

// required for hot reloading
#[unsafe(no_mangle)]
pub fn handle_input(_: glazer::PlatformInput<Acid>) {}

#[unsafe(no_mangle)]
pub fn update_and_render(
    glazer::PlatformUpdate {
        memory: da,
        sample_rate,
        samples,
        channels,
        frame_buffer,
        width,
        height,
        reloaded,
        ..
    }: glazer::PlatformUpdate<Acid, u32>,
) {
    for y in 0..height as u32 {
        for x in 0..width as u32 {
            let red = x % 255;
            let green = y % 255;
            let blue = (x * y) % 255;
            let index = y as usize * width + x as usize;
            frame_buffer[index] = blue | (green << 8) | (red << 16);
        }
    }
    if reloaded {
        da.rebuild_sounds();
    }
    da.process(samples, sample_rate, channels);
}
