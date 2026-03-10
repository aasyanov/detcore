use crate::*;

#[derive(Clone)]
pub struct IndustrialState {
    pub last_seq: Seq,
    pub pressure: Decimal,
    pub pump_active: bool,
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

#[derive(Clone, Copy)]
pub enum EventKind {
    PressureUpdate { tag_id: u32, value: Decimal },
    EmergencyStop,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CommandKind {
    StopPump,
    TriggerAlarm { id: u32 },
}

pub struct IndustrialLogic;

impl Logic<IndustrialState, EventKind, CommandKind> for IndustrialLogic {
    fn step(
        state: &mut IndustrialState,
        event: Event<EventKind>,
        commands: &mut Vec<Command<CommandKind>, 16>,
    ) {
        match event.payload {
            EventKind::PressureUpdate { tag_id, value } => {
                if tag_id == 1 {
                    state.pressure = value;
                    if value > 100 * SCALE {
                        let _ = commands.push(Command::Emit(CommandKind::StopPump));
                        let _ = commands.push(Command::Emit(CommandKind::TriggerAlarm { id: 101 }));
                        state.pump_active = false;
                    }
                }
            }
            EventKind::EmergencyStop => {
                let _ = commands.push(Command::Emit(CommandKind::StopPump));
                state.pump_active = false;
            }
        }
    }
}
