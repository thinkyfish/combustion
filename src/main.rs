#![allow(unknown_lints, unused_imports, dead_code, inline_always, toplevel_ref_arg)]
#![feature(proc_macro, receiver_try_iter, specialization)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate enum_primitive;
extern crate libc;
extern crate nalgebra;
extern crate rusttype;
extern crate image;
extern crate palette;
extern crate num_traits;
extern crate regex;
extern crate assimp;
extern crate glfw;
extern crate nice_glfw;
extern crate time;
extern crate specs;
extern crate num_cpus;
extern crate vec_map;
extern crate petgraph;
extern crate lazy;

use std::thread;
use std::sync::mpsc;

use glfw::{Glfw, Action, Context, Key, WindowHint, WindowEvent};

#[macro_use]
pub mod macros;

pub mod error;
pub mod utils;
pub mod preprocessor;

#[macro_use]
pub mod backend;

#[macro_use]
pub mod components;
pub mod resources;
pub mod entities;
pub mod systems;

pub mod storage;
pub mod scene;
pub mod graphics;
pub mod game;

use graphics::{RenderSignal, FullscreenToggle};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = nice_glfw::WindowBuilder::new(&mut glfw)
        .try_modern_context_hints()
        .size(1280, 720)
        //.aspect_ratio(16, 9)
        .common_hints(&[
            WindowHint::Visible(true),
            //WindowHint::Samples(Some(2)),
            WindowHint::DoubleBuffer(true),
            WindowHint::OpenGlDebugContext(true),
        ])
        .title("OpenGL Testing")
        .create()
        .expect("Couldn't create window");

    //Enable interactivity
    window.set_all_polling(true);

    //Load up all the OpenGL functions from the process
    backend::gl::bindings::load_all_with(|symbol| window.get_proc_address(symbol) as *const _);

    //Enable debugging of OpenGL messages
    backend::gl::enable_debug(backend::gl::default_debug_callback, true).unwrap();

    //Create channel for forwarding events to the render thread
    let (tx, rx) = mpsc::channel();

    // Disconnect current context
    glfw::make_context_current(None);

    //Create Send-able context to send to render thread
    let context = window.render_context();

    //Start render thread
    let render_thread: thread::JoinHandle<_> = thread::Builder::new().name("Render thread".to_string()).spawn(move || {
        use graphics::render::RenderLoopState;

        println!("Render thread started...");

        //Make the OpenGL context active on the render thread
        glfw::make_context_current(Some(&context));

        let mut state: RenderLoopState = RenderLoopState::new(60.0);

        state.unpause();

        graphics::render::start(&mut state, context, rx).expect("Render thread crashed");

        //Once rendering has ended, free the OpenGL context
        glfw::make_context_current(None);

        println!("Finished after {} frames", state.total_frames());
    }).expect("Could not create Render thread");

    //Create fullscreen toggle in primary thread
    let mut fullscreen = FullscreenToggle::new();

    macro_rules! send_and_unpark {
        ($event:expr) => ({
            let ret = tx.send($event);
            render_thread.thread().unpark();
            ret
        })
    }

    //Since the primary thread will do nothing but wait on events, do that
    while !window.should_close() {
        //Instead of polling, actively block the thread since nothing else is happening in it
        glfw.wait_events();

        //While most events are simply forwarded to the
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                WindowEvent::Key(Key::F11, _, Action::Press, _) => {
                    fullscreen.toggle(&mut glfw, &mut window);
                }
                WindowEvent::FramebufferSize(width, height) |
                WindowEvent::Size(width, height) if width > 0 && height > 0 => {
                    send_and_unpark!(RenderSignal::ViewportResize(width, height)).unwrap();
                }
                WindowEvent::Iconify(iconified) if iconified => {
                    send_and_unpark!(RenderSignal::Pause).unwrap();
                }
                WindowEvent::Focus(focus) => {
                    if focus {
                        send_and_unpark!(RenderSignal::Resume).unwrap();
                    } else {
                        send_and_unpark!(RenderSignal::Pause).unwrap();
                    }
                }
                _ => {
                    tx.send(RenderSignal::Event(event)).unwrap();
                }
            }
        }
    }

    println!("Shutting down...");

    //Signal the render thread to close
    send_and_unpark!(RenderSignal::Stop).expect("Failed to signal render task.");

    render_thread.join().expect("Failed to join render thread");
}