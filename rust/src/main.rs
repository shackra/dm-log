use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Rect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "x11")]
    std::env::set_var("SDL_VIDEO_DRIVER", "x11");

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl3::ttf::init()?;

    let window = video_subsystem
        .window("mazaforja", 800, 600)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();

    // Load a font - try common system paths
    let font_path = find_font()?;
    let font = ttf_context.load_font(&font_path, 32.0)?;

    let surface = font.render("hello world").blended(Color::WHITE)?;
    let texture = texture_creator.create_texture_from_surface(&surface)?;

    let text_width = surface.width();
    let text_height = surface.height();
    let text_rect = Rect::new(
        (800 - text_width as i32) / 2,
        (600 - text_height as i32) / 2,
        text_width,
        text_height,
    );

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape | Keycode::Q),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.copy(&texture, None, text_rect)?;
        canvas.present();
    }

    Ok(())
}

fn find_font() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let fc = fontconfig::Fontconfig::new().ok_or("Failed to init fontconfig")?;
    let font = fc
        .find("sans-serif", None)
        .ok_or("No sans-serif font found via fontconfig")?;
    Ok(font.path)
}
