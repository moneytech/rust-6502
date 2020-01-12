extern crate cursive;
use std::env;

use std::fs;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;

mod computer;

use computer::{Processor, Computer, ControllerMessage};

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
    ui_tx: mpsc::Sender<UiMessage>,
    controller_tx: mpsc::Sender<ControllerMessage>,
}

pub enum UiMessage {
    UpdateProcessor(Processor),
    UpdateData(Vec<u8>),
}

impl Ui {
    /// Create a new Ui object.  The provided `mpsc` sender will be used
    /// by the UI to send messages to the controller.
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>) -> Ui {
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let mut ui = Ui {
            cursive: Cursive::default(),
            ui_tx: ui_tx,
            ui_rx: ui_rx,
            controller_tx: controller_tx,
        };

        // Create a view tree with a TextArea for input, and a
        // TextView for output.
        let controller_tx_clone = ui.controller_tx.clone();
        let controller_tx_clone1 = ui.controller_tx.clone();
        let controller_tx_clone2 = ui.controller_tx.clone();
        let controller_tx_clone3 = ui.controller_tx.clone();
        ui.cursive.add_layer(
            Dialog::around(
                LinearLayout::horizontal()
                .child(Dialog::around(
                    TextView::new("TEST MEM").with_id("memory")
                ).title("Memory"))
                .child(
                    LinearLayout::vertical()
                    .child(Dialog::around(
                        TextView::new("PROC DATA").with_id("processor")
                    ).title("Processor info"))
                    .child(Dialog::around(
                        TextView::new("PROC INFO").with_id("info")
                    ).title("Debug info").scrollable())
                )
            )
            
            .button("Faster", move |s| {
                controller_tx_clone.send(
                    ControllerMessage::ButtonPressed("faster".to_string())
                )
                .unwrap();
            })
            .button("Slower", move |s| {
                controller_tx_clone1.send(
                    ControllerMessage::ButtonPressed("slower".to_string())
                )
                .unwrap();
            })
            .button("Pause", move |s| {
                controller_tx_clone2.send(
                    ControllerMessage::ButtonPressed("pause".to_string())
                )
                .unwrap();
            })
            .button("Step", move |s| {
                controller_tx_clone3.send(
                    ControllerMessage::ButtonPressed("step".to_string())
                )
                .unwrap();
            })
            .button("Quit", |s| {
                std::process::abort();
                std::process::exit(0);
            })
            .title("6502 simulator")
            .full_screen()
        );

        // Configure a callback
        
        
        ui
    }

    /// Step the UI by calling into Cursive's step function, then
    /// processing any UI messages.
    pub fn step(&mut self) -> bool {
        if !self.cursive.is_running() {
            return false;
        }

        // Process any pending UI messages
        while let Some(message) = self.ui_rx.try_iter().next() {
            match message {
                UiMessage::UpdateProcessor(processor) => {
                    //println!("UpdateProcessor {}", processor.clock);
                    let mut output = self.cursive
                        .find_id::<TextView>("processor")
                        .unwrap();
                    output.set_content(format!("{:?}", processor));

                    let mut info = self.cursive
                        .find_id::<TextView>("info")
                        .unwrap();
                    let fmt = format!("{}\n{}", processor.info, info.get_content().source());
                    info.set_content(fmt);
                },
                UiMessage::UpdateData(data) => {
                    let mut output = self.cursive
                        .find_id::<TextView>("memory")
                        .unwrap();
                    output.set_content(format!("{:x?}", data));
                },
            }
        }

        // Step the UI
        self.cursive.step();
        self.cursive.refresh();
        true
    }
}


pub struct Controller {
    rx: mpsc::Receiver<ControllerMessage>,
    ui: Ui,
    computer: Computer,
}

impl Controller {
    /// Create a new controller
    pub fn new(filename: String) -> Result<Controller, String> {
        let data = fs::read(filename).expect("could not read file");
        let (tx, rx) = mpsc::channel::<ControllerMessage>();
        Ok(Controller {
            rx: rx,
            ui: Ui::new(tx.clone()),
            computer: Computer::new(tx.clone(), data),
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        let mut speed = 24;
        let mut i = 1;
        let mut paused: bool = true;
        let mut step: bool = false;
        while self.ui.step() {
            if (i % speed == 0 && !paused) || (paused && step) {
                self.computer.step();
                step = false;
            }

            i += 1;
            
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    ControllerMessage::ButtonPressed(btn) => {
                        if btn == "faster" && speed >= 2 {
                            speed /= 2;
                        } else if btn == "slower" && speed <= 1000 {
                            speed *= 2;
                        } else if btn == "pause" && speed <= 1000 {
                            paused = !paused;
                        } else if btn == "step" && speed <= 1000 {
                            step = true;
                        }
                    },
                    ControllerMessage::UpdatedProcessorAvailable(processor) => {
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateProcessor(processor))
                            .unwrap();
                        //self.computer.step();
                    },
                    ControllerMessage::UpdatedDataAvailable(data) => {
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateData(data))
                            .unwrap();
                    },
                };
            }
        }
    }
}

fn main() {
    // Launch the controller and UI
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please enter a filename to run");
    }
    let filename = &args[1];

    let controller = Controller::new(filename.to_string());
    match controller {
        Ok(mut controller) => controller.run(),
        Err(e) => println!("Error: {}", e),
    };
}