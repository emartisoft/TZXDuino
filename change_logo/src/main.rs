use fltk::{
    app,
    button::Button,
    enums::{Color, Event, Shortcut},
    frame::Frame,
    menu::{MenuBar, MenuFlag},
    prelude::*,
    window::Window,
};
use image::ImageReader;
use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Write},
    rc::Rc,
};

const WIDTH: usize = 128;
const HEIGHT: usize = 32;
const SCALE: i32 = 4;

fn main() {
    let app = app::App::default();

    let mut win = Window::new(
        100,
        100,
        WIDTH as i32 * SCALE + 20,
        HEIGHT as i32 * SCALE + 90,
        "OLED SSD1306 Logo Editor for TZXDuino",
    );

    let pixels = Rc::new(RefCell::new(vec![false; WIDTH * HEIGHT]));

    /* ---------------- CANVAS ---------------- */

    let canvas = Rc::new(RefCell::new(Frame::new(
        10,
        40,
        WIDTH as i32 * SCALE,
        HEIGHT as i32 * SCALE,
        "",
    )));

    /* ---------------- MENU ---------------- */

    let mut menu = MenuBar::new(0, 0, win.w(), 30, "");

    {
        let p = pixels.clone();
        let c = canvas.clone();
        menu.add(
            "&File/Open &BMP",
            Shortcut::None,
            MenuFlag::MenuDivider,
            move |_| load_bmp(&p, &c),
        );
    }

    {
        let p = pixels.clone();
        let c = canvas.clone();
        menu.add(
            "&File/&Open OLED",
            Shortcut::None,
            MenuFlag::Normal,
            move |_| load_oled(&p, &c),
        );
    }

    {
        let p = pixels.clone();
        menu.add(
            "&File/&Save OLED",
            Shortcut::None,
            MenuFlag::MenuDivider,
            move |_| save_oled(&p),
        );
    }

    {
        let p = pixels.clone();
        menu.add(
            "&File/&Export customlogo.h",
            Shortcut::None,
            MenuFlag::MenuDivider,
            move |_| export_customlogo_h(&p),
        );
    }

    // ---------- Exit (horizontal line + item) ----------
    {
        menu.add("&File/E&xit", Shortcut::None, MenuFlag::Normal, move |_| {
            app.quit()
        });
    }

    // ---------- About under Help ----------
    {
        menu.add(
            "&Help/&About",
            Shortcut::None,
            MenuFlag::Normal,
            |_| {
                fltk::dialog::message_default(
                    "OLED SSD1306 Logo Editor for TZXDuino\nVersion: 1.0.0\n\nCoded by emarti, Murat Özdemir",
                );
            },
        );
    }

    /* ---------------- DRAW ---------------- */

    {
        let pixels = pixels.clone();
        let canvas = canvas.clone();
        canvas.borrow_mut().draw(move |f| {
            let p = pixels.borrow();

            fltk::draw::set_draw_color(Color::Black);
            fltk::draw::draw_rectf(f.x(), f.y(), f.w(), f.h());

            fltk::draw::set_draw_color(Color::White);
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if p[y * WIDTH + x] {
                        fltk::draw::draw_rectf(
                            f.x() + x as i32 * SCALE,
                            f.y() + y as i32 * SCALE,
                            SCALE,
                            SCALE,
                        );
                    }
                }
            }
        });
    }

    /* ---------------- HANDLE ---------------- */

    {
        let pixels = pixels.clone();
        let canvas = canvas.clone();
        canvas.borrow_mut().handle(move |f, ev| match ev {
            Event::Push | Event::Drag => {
                let mx = (app::event_x() - f.x()) / SCALE;
                let my = (app::event_y() - f.y()) / SCALE;

                if mx >= 0 && my >= 0 && mx < WIDTH as i32 && my < HEIGHT as i32 {
                    let idx = my as usize * WIDTH + mx as usize;
                    let mut p = pixels.borrow_mut();

                    // Left click => set true
                    if app::event_button() == 1 {
                        p[idx] = true;
                    }
                    // Right click => set false
                    else if app::event_button() == 3 {
                        p[idx] = false;
                    }

                    f.redraw();
                }
                true
            }
            _ => false,
        });
    }

    /* ---------------- CLEAR ---------------- */

    let mut clear = Button::new(10, win.h() - 35, 80, 25, "Clear");
    {
        let pixels = pixels.clone();
        let canvas = canvas.clone();
        clear.set_callback(move |_| {
            pixels.borrow_mut().fill(false);
            canvas.borrow_mut().redraw();
        });
    }

    win.end();
    win.show();
    app.run().unwrap();
}

/* ================= BMP IMPORT ================= */

fn load_bmp(pixels: &Rc<RefCell<Vec<bool>>>, canvas: &Rc<RefCell<Frame>>) {
    let path = fltk::dialog::file_chooser("Open BMP", "*.bmp", ".", false);
    let Some(path) = path else { return };

    let img = ImageReader::open(path).unwrap().decode().unwrap().to_rgb8();

    let (w, h) = img.dimensions();
    let mut p = pixels.borrow_mut();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let sx = x as u32 * w / WIDTH as u32;
            let sy = y as u32 * h / HEIGHT as u32;
            let px = img.get_pixel(sx, sy);
            let lum = (px[0] as u16 + px[1] as u16 + px[2] as u16) / 3;
            p[y * WIDTH + x] = lum > 128;
        }
    }

    canvas.borrow_mut().redraw();
}

/* ================= OLED SAVE ================= */

fn save_oled(pixels: &Rc<RefCell<Vec<bool>>>) {
    let path = fltk::dialog::file_chooser("Save OLED", "*.oled", ".", true);
    let Some(path) = path else { return };

    let p = pixels.borrow();
    let mut out = Vec::new();

    for page in 0..HEIGHT / 8 {
        for x in 0..WIDTH {
            let mut byte = 0u8;
            for bit in 0..8 {
                let y = page * 8 + bit;
                if p[y * WIDTH + x] {
                    byte |= 1 << bit;
                }
            }
            out.push(byte);
        }
    }

    File::create(path).unwrap().write_all(&out).unwrap();
}

/* ================= OLED LOAD ================= */

fn load_oled(pixels: &Rc<RefCell<Vec<bool>>>, canvas: &Rc<RefCell<Frame>>) {
    let path = fltk::dialog::file_chooser("Open OLED", "*.oled", ".", false);
    let Some(path) = path else { return };

    let mut data = Vec::new();
    File::open(path).unwrap().read_to_end(&mut data).unwrap();

    let mut p = pixels.borrow_mut();
    let mut i = 0;

    for page in 0..HEIGHT / 8 {
        for x in 0..WIDTH {
            let byte = data[i];
            i += 1;
            for bit in 0..8 {
                let y = page * 8 + bit;
                p[y * WIDTH + x] = (byte & (1 << bit)) != 0;
            }
        }
    }

    canvas.borrow_mut().redraw();
}

/* ================= customlogo.h EXPORT ================= */

fn export_customlogo_h(pixels: &Rc<RefCell<Vec<bool>>>) {
    let path = fltk::dialog::file_chooser("Export customlogo.h", "*.h", ".", true);
    let Some(path) = path else { return };

    let p = pixels.borrow();
    let mut out = String::new();

    out.push_str("// Generated by OLED SSD1306 Logo Editor for TZXDuino\n");
    out.push_str("const byte logo [] PROGMEM = {\n");

    let mut first = true;
    let mut count = 0;

    for page in 0..HEIGHT / 8 {
        for x in 0..WIDTH {
            let mut byte = 0u8;
            for bit in 0..8 {
                let y = page * 8 + bit;
                if p[y * WIDTH + x] {
                    byte |= 1 << bit;
                }
            }

            if !first {
                out.push_str(", ");
            }
            first = false;

            out.push_str(&format!("0x{:02X}", byte));

            count += 1;
            if count % 8 == 0 {
                out.push('\n');
            }
        }
    }

    out.push_str("};\n");

    File::create(path)
        .unwrap()
        .write_all(out.as_bytes())
        .unwrap();
}
