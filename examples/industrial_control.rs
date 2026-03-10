use detcore::{Command, Engine, Event, Logic, Seq, State, Timestamp, Vec, SCALE};

#[derive(Clone)]
struct IndustrialState {
    last_seq: Seq,
    pressure: i64,
    pump_active: bool,
}

impl State for IndustrialState {
    fn check_invariants(&self) -> bool {
        self.pressure >= 0
    }

    fn last_seq(&self) -> Seq {
        self.last_seq
    }

    fn set_last_seq(&mut self, seq: Seq) {
        self.last_seq = seq;
    }
}

impl Default for IndustrialState {
    fn default() -> Self {
        Self {
            last_seq: Seq(0),
            pressure: 0,
            pump_active: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PressureEvent {
    SensorReading { pressure_bar: i64 },
    EmergencyStop,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ControlCommand {
    StopPump,
    StartPump,
    TriggerAlarm { severity: u8 },
}

struct PressureController;

impl Logic<IndustrialState, PressureEvent, ControlCommand> for PressureController {
    fn step(
        state: &mut IndustrialState,
        event: Event<PressureEvent>,
        commands: &mut Vec<Command<ControlCommand>, 16>,
    ) {
        match event.payload {
            PressureEvent::SensorReading { pressure_bar } => {
                state.pressure = pressure_bar * SCALE;

                if pressure_bar > 100 {
                    let _ = commands.push(Command::Emit(ControlCommand::StopPump));
                    let _ =
                        commands.push(Command::Emit(ControlCommand::TriggerAlarm { severity: 2 }));
                    state.pump_active = false;
                } else if pressure_bar < 20 && !state.pump_active {
                    let _ = commands.push(Command::Emit(ControlCommand::StartPump));
                    state.pump_active = true;
                }
            }
            PressureEvent::EmergencyStop => {
                let _ = commands.push(Command::Emit(ControlCommand::StopPump));
                let _ = commands.push(Command::Emit(ControlCommand::TriggerAlarm { severity: 3 }));
                state.pump_active = false;
            }
        }
    }
}

fn main() {
    println!("detcore: Industrial Pressure Controller");
    println!("=======================================\n");

    let mut ctrl =
        Engine::<IndustrialState, PressureController, PressureEvent, ControlCommand>::new(
            IndustrialState::default(),
        );

    let events = vec![
        (
            Seq(1),
            Timestamp(1000),
            PressureEvent::SensorReading { pressure_bar: 50 },
        ),
        (
            Seq(2),
            Timestamp(2000),
            PressureEvent::SensorReading { pressure_bar: 120 },
        ),
        (
            Seq(3),
            Timestamp(3000),
            PressureEvent::SensorReading { pressure_bar: 10 },
        ),
        (Seq(4), Timestamp(4000), PressureEvent::EmergencyStop),
    ];

    for (seq, ts, payload) in events {
        let commands = ctrl.process(Event { seq, ts, payload });

        println!("Event:  {:?}", payload);
        println!(
            "State:  pressure={:.2} bar, pump={}",
            ctrl.state.pressure as f64 / SCALE as f64,
            if ctrl.state.pump_active { "ON" } else { "OFF" }
        );

        if commands.is_empty() {
            println!("Output: (none)");
        } else {
            for cmd in &commands {
                match cmd {
                    Command::Emit(ControlCommand::StopPump) => println!("Output: STOP PUMP"),
                    Command::Emit(ControlCommand::StartPump) => println!("Output: START PUMP"),
                    Command::Emit(ControlCommand::TriggerAlarm { severity }) => {
                        println!("Output: ALARM severity={severity}")
                    }
                    Command::NoOp => {}
                }
            }
        }
        println!();
    }

    println!("Invariants OK: {}", ctrl.state.check_invariants());
}
