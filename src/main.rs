use gif::DecodeOptions;
use image::{DynamicImage, ImageBuffer, RgbaImage};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};
use ratatui_image::{
    FilterType, StatefulImage,
    picker::{Capability, Picker},
    protocol::StatefulProtocol,
};
use std::{
    fs::File,
    io::{self},
};

use crossterm::event;
use crossterm::execute;
use crossterm::{
    event::{Event, KeyCode},
    terminal::{EnterAlternateScreen, enable_raw_mode},
};

const ZOOM: u32 = 2;

fn decode_gif(f: &mut File) -> Result<Vec<RgbaImage>, Box<dyn std::error::Error>> {
    let mut opts: DecodeOptions = DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);

    // let mut decoder = opts.read_info(f)?;

    let mut decoder = match opts.read_info(f) {
        Ok(d) => d,
        Err(e) => {
            return Err(e.into());
            // return Err(Box::new(Error::new(ErrorKind::NotFound, e)));
        }
    };

    let mut images: Vec<RgbaImage> = Vec::new();

    while let Some(frame) = decoder.read_next_frame()? {
        let frame_width = frame.width as u32;
        let frame_height = frame.height as u32;
        println!("Frame {}*{}", frame_width, frame_height);
        let buf = &frame.buffer;
        let img: RgbaImage =
            ImageBuffer::from_raw(frame_width, frame_height, buf.to_vec()).unwrap();
        images.push(img);
    }

    Ok(images)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::open("reload.gif")?;

    let rgba_images: Vec<RgbaImage> = decode_gif(&mut f)?;
    // let picker = Picker::halfblocks();
    let picker = Picker::from_query_stdio()?;

    let mut protocols: Vec<StatefulProtocol> = rgba_images
        .into_iter()
        .map(DynamicImage::ImageRgba8)
        .map(|d| d.resize(d.width() * ZOOM, d.height() * ZOOM, FilterType::Nearest))
        .map(|d| picker.new_resize_protocol(d))
        .collect();

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut exit: bool = false;
    while !exit {
        for p in protocols.iter_mut() {
            terminal.draw(|f| ui_r(f, p))?;
            #[allow(clippy::collapsible_if)]
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        exit = true;
                        break;
                    }
                }
            }
        }
    }

    let caps: &Vec<Capability> = Picker::capabilities(&picker);
    caps.iter().for_each(|c| print!("{:?} ", c));

    Ok(())
}

#[allow(dead_code)]
fn ui(f: &mut Frame<'_>, image: &mut StatefulProtocol) {
    let widget = StatefulImage::default();
    f.render_stateful_widget(widget, f.area(), image);
}

fn ui_r(f: &mut Frame<'_>, image: &mut StatefulProtocol) {
    let area = f.area();

    // On crée le widget avec une configuration explicite
    let widget = StatefulImage::default()
        // On force le redimensionnement au niveau du Widget
        .resize(ratatui_image::Resize::Fit(Some(FilterType::Lanczos3)));

    // On passe l'image (le protocole)
    // C'est f.render_stateful_widget qui est censé appeler
    // les calculs de taille internes via size_for
    f.render_stateful_widget(widget, area, image);
}

// fn ui_r(f: &mut Frame<'_>, image: &mut StatefulProtocol) {
//     // 1. Créer le widget
//     let widget = StatefulImage::default()
//         // .resize(Resize::Fit) redimensionne l'image pour qu'elle tienne
//         // dans la zone tout en gardant ses proportions (évite de l'écraser)
//         .resize(ratatui_image::Resize::Fit(Some(FilterType::Lanczos3)));

//     // 2. Définir la zone (ici, tout l'écran)
//     let area = f.area();

//     // 3. Rendre le widget
//     f.render_stateful_widget(widget, area, image);
// }
