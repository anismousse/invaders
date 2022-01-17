use crossterm::{cursor::Show, cursor::Hide, ExecutableCommand, event::{self, Event, KeyCode}};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use invaders::frame::new_frame;
use rusty_audio::Audio;
use std::{error::Error, time::{Duration, Instant}, sync::mpsc};
use std::{io, thread};
use invaders::{render, frame, player::Player, frame::Drawable};

fn main() -> Result<(), Box<dyn Error>>{
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("gameover", "game_over.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("win", "win.wav");
    audio.add("startup", "startup.wav");
    audio.play("explode");

    //Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen);
    stdout.execute(Hide)?;
    
    //Render loop in different thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = std::thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame= match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });
    // Game Loop

    let mut player = Player::new();
    let mut instant = Instant::now();
    'gameloop: loop{
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame =  new_frame();
        //input
        while event::poll(Duration::default())?{
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter=> {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') =>{
                        audio.play("gameover");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }
        //upda
        player.update(delta);
        // Draw & render
        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen);
    terminal::disable_raw_mode()?;
    Ok(())
}
