
use midi_msg::{MidiMsg, ReceiverContext, SystemRealTimeMsg};
use midir::{MidiInput, Ignore, MidiInputPort};
use minifb::{Key, MouseButton, MouseMode};

use std::io::Write;
use std::error::Error;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;


mod chord;
mod drum;
mod text;

mod midi;
use crate::midi::MidiProcessor;

mod win;
use crate::win::*;

mod graphics;
use crate::graphics::RGB;

mod gui;
use crate::gui::{KeyboardFocus, Inst, Direction};

mod setup;
use crate::setup::Setup;


fn main() -> Result<(), Box<dyn Error>> {
    let mut setup = Setup::load("weirdgrv.json")?;

    let mut midi_in = MidiInput::new("midir reading input").unwrap();
    midi_in.ignore(Ignore::None);

    //let in_port = get_in_port(&midi_in).unwrap();
    let in_port = &midi_in.ports()[setup.midi_port_id];
    let in_port_name = midi_in.port_name(&in_port).unwrap();
    println!("Connection open, reading input from '{}' ...", in_port_name);

    let (tx, rx): (Sender<MidiMsg>, Receiver<MidiMsg>) = mpsc::channel();

    let mut ctx = ReceiverContext::new();
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(&in_port, "midir-read-input", move |_stamp, message, _| {
        let thread_tx = tx.clone();
        let (msg, _len) = MidiMsg::from_midi_with_context(&message, &mut ctx).expect("Not an error");
        if let MidiMsg::SystemRealTime{ msg: SystemRealTimeMsg::TimingClock } = msg {
            // no-op
        } else {
            thread_tx.send(msg).expect("failed to tx message");
        }
        
    }, ())?;

    let mut win = Win::init(640, 480);

    let mut focus = KeyboardFocus::new();

    while win.window.is_open() && win.running {
        win.clear();
        for msg in rx.try_iter() {
            for chord in &mut setup.chords {
                chord.deal_with(msg.clone());
            }
            setup.kit.deal_with(msg.clone());
            for text in &mut setup.texts {
                text.deal_with(msg.clone())
            }
        }

        for chord in &mut setup.chords {
            win.draw_chord(chord, setup.global_transpose);
        }
        for drum in &mut setup.kit.drums {
            win.draw_drum(drum);
        }
        for text in &mut setup.texts {
            win.draw_text(text);
        }
        if focus.editing {
            win.draw_editor(&setup, &focus);
        }
        win.draw_status_message();
        
        // keyboard time
        win.window.get_keys_pressed(minifb::KeyRepeat::Yes).iter().for_each(|key|
            match key {
                Key::Enter => focus.editing = !focus.editing,
                Key::A => focus.preview_all = !focus.preview_all,
                Key::C => focus.inst = Inst::Chord,
                Key::D => focus.inst = Inst::Drum,
                Key::Key1 | Key::Key2 | Key::Key3 | Key::Key4 | Key::Key5
                | Key::Key6 | Key::Key7 | Key::Key8 | Key::Key9 | Key::Key0 => {
                    let val = if *key == Key::Key0 {10} else {*key as u8};
                    focus.index = if win.window.is_key_down(Key::LeftShift) {
                        val - 1 + 10
                    } else {
                        val - 1
                    };
                }
                Key::P => {
                    dbg!(&focus);
                    win.set_status_message("test message", RGB::fff());
                }
                Key::Right | Key::Left | Key::Up | Key::Down => {
                    focus.adjustment(
                        &mut setup, 
                        Direction::from_key(key).unwrap(), 
                        win.window.is_key_down(Key::LeftShift),
                        win.window.is_key_down(Key::LeftCtrl),
                    );
                }

                Key::R => {
                    // reload setup
                    let new_setup = Setup::load(&setup.path);
                    match new_setup {
                        Ok(s) => {
                            win.set_status_message(&format!("Reloaded setup: {}", &setup.path), RGB::fff());
                            setup = s;
                        },
                        Err(e) => {
                            win.set_status_message("err see console", RGB::color("red"));
                            println!("error parsing setup.. {}", e);
                        }
                    }
                }
                Key::S => {
                    match setup.maybe_write() {
                        Ok(data_length) => {
                            win.set_status_message(&format!("Saved setup: {} ({}c)", &setup.path, data_length), RGB::fff());
                        },
                        Err(e) => {
                            win.set_status_message("err see console", RGB::color("red"));
                            println!("error writing: {}", e)
                        },
                    }
                }
                Key::Q => {
                    win.running = false;
                },
                _ => (),
            }
        );

        // mouse support
        if win.window.get_mouse_down(MouseButton::Left) {
            let pos = win.window.get_mouse_pos(MouseMode::Clamp).unwrap();
            focus.mouse_adjustment(
                &mut setup,
                pos.0,
                pos.1
            );
        }

        win.update();
    }

    // end
    Ok(())
}

/// Get an input port (read from console if multiple are available)
fn get_in_port(midi_in: &MidiInput) -> Result<MidiInputPort, Box<dyn Error>> {    
    let in_ports = midi_in.ports();
    //in_ports = vec![in_ports.remove(0)];

    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let i = input.trim().parse::<usize>()?;
            in_ports.get(i).unwrap()
        }
    };
    Ok(in_port.clone())
}