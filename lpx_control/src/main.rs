//! Use the MIDI control keys from the LPX to run programmes.
// use std::io::stdin;
use midi_connection::MIDICommunicator;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// Colours used for the keys to provide feedback
static ENABLEDCOLOUR: u8 = 87; // Ready
static DISABLEDCOLOUR: u8 = 5; // Disabled
static SELECTEDCOLOUR: u8 = 67; // In use

// The number of seconds to make the controls inactive when
// notes played
static SLEEPDURATION: usize = 2;

/// Dispatcher matches a control key to an executable and executes it
struct Dispatcher {
    // Associate a control value with a command. When a command runs
    // the previous command stops
    up_table: HashMap<u8, String>,
    down_table: HashMap<u8, String>,

    // The last pad pressed.  At the start this is None
    last: Option<u8>,
}
impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
impl Dispatcher {
    fn new() -> Self {
        // Function tables for control keys on LPX.  Keys are the MIDI
        // bytes passed from the pad and the values are the commands,
        // as String.  A command string refers to an executable file
        // in <PWD>/subs/.  Not every control key needs to be here,
        // and if an executable does not exist it is ignored.

        // Each button on the device has two commands associated with
        // it: Firstly when it is activated.  Second when a button is
        // pressed after it has been activated.  "ON" and "OFF".

        // The commands for each button (executable files under
        // <PWD>/subs/) are named: ON-Ctl.N and OFF-Ctl-N where N is
        // MIDI value.
        let mut up_table: HashMap<u8, String> = HashMap::new();
        let mut down_table: HashMap<u8, String> = HashMap::new();

        // When a button pressed store its MIDI value here
        let last: Option<u8> = None;

        up_table.insert(19, "ON-CTL.19".to_string());
        up_table.insert(29, "ON-CTL.29".to_string());
        up_table.insert(39, "ON-CTL.39".to_string());
        up_table.insert(49, "ON-CTL.49".to_string());
        up_table.insert(59, "ON-CTL.59".to_string());
        up_table.insert(69, "ON-CTL.69".to_string());
        up_table.insert(79, "ON-CTL.79".to_string());
        up_table.insert(89, "ON-CTL.89".to_string());

        down_table.insert(19, "OFF-CTL.19".to_string());
        down_table.insert(29, "OFF-CTL.29".to_string());
        down_table.insert(39, "OFF-CTL.39".to_string());
        down_table.insert(49, "OFF-CTL.49".to_string());
        down_table.insert(59, "OFF-CTL.59".to_string());
        down_table.insert(69, "OFF-CTL.69".to_string());
        down_table.insert(79, "OFF-CTL.79".to_string());
        down_table.insert(89, "OFF-CTL.89".to_string());

        Self {
            down_table: down_table,
            up_table: up_table,
            last: last,
        }
    }

    /// `run_cmd` is called when the pad/key has been processed.
    /// `cmd` is a path to an executable
    fn run_cmd(cmd: &str) {
        // eprintln!("run_cmd({}) Starts", &cmd);
        let mut one_20_proof_home: String = ".".to_string();
        for (key, value) in env::vars() {
            if key == "Home120Proof" {
                one_20_proof_home = value;
                break;
            }
        }
        let home_dir = format!("{}/subs", &one_20_proof_home);
        let home_dir = Path::new(home_dir.as_str());
        env::set_current_dir(&home_dir)
            .expect(format!("Cannot change directory to: {}", home_dir.display()).as_str());
        let command = format!("{}/{}", home_dir.display(), &cmd);
        // eprintln!("run_cmd({}) Command: {}", &cmd, &command);
        match process::Command::new(command.as_str()).output() {
            Ok(out) => {
                if out.status.success() {
                    // let s = String::from_utf8_lossy(&out.stdout);
                    eprintln!("Success: {} ", cmd)
                } else {
                    let s = String::from_utf8_lossy(&out.stderr);
                    eprintln!("Not success: {} and stderr was:{}", cmd, s)
                }
            }
            Err(err) => eprintln!("Failure: cmd {}  Err: {:?}", command, err),
        }
        // eprintln!("run_cmd({}) Ends", &cmd);
    }

    /// A control pad has been pressed
    /// `ctl` is the pad number
    fn run_ctl(&mut self, ctl: u8, lpx_midi: Arc<Mutex<MIDICommunicator<()>>>) {
        // Shut down the last control used
        // eprintln!("run_ctl({}) Starts", ctl);
        if let Some(x) = self.last {
            // eprintln!("There was a last: {}", &x);
            match self.down_table.get(&x) {
                Some(cmd) => {
                    // There is a command to run for shutting down last control

                    // Start flashing the selected pad to illustrate it is turning off
                    let out_message_flash: [u8; 11] =
                        [240, 0, 32, 41, 2, 12, 3, 2, x, SELECTEDCOLOUR, 247];
                    let mut midi_comm = lpx_midi.lock().unwrap();
                    match midi_comm.send(&out_message_flash) {
                        Ok(()) => (), //eprintln!("Sent message: {:?}", &out_message_flash),
                        Err(err) => eprintln!("Failed send: {:?}", err),
                    };

                    Self::run_cmd(cmd.as_str());

                    // Colour the pad enabled
                    let out_message_disable: [u8; 11] =
                        [240, 0, 32, 41, 2, 12, 3, 0, x, ENABLEDCOLOUR, 247];
                    match midi_comm.send(&out_message_disable) {
                        Ok(()) => (), //eprintln!("Sent message: {:?}", &out_message_disable),
                        Err(err) => eprintln!("Failed send: {:?}", err),
                    };
                }
                // The last control does not need anything special to
                // shutdown
                None => (),
            }
        }
        self.last = Some(ctl);

        match self.up_table.get(&ctl) {
            Some(cmd) => {
                // eprintln!("run_ctl({}) Run command: {}", ctl, &cmd);
                // Flash pad to show it is being enabled
                let out_message_flash: [u8; 11] =
                    [240, 0, 32, 41, 2, 12, 3, 2, ctl, SELECTEDCOLOUR, 247];
                let mut midi_comm = lpx_midi.lock().unwrap();
                match midi_comm.send(&out_message_flash) {
                    Ok(()) => (), //eprintln!("Sent message: {:?}", &out_message_flash),
                    Err(err) => eprintln!("Failed send: {:?}", err),
                };

                Self::run_cmd(cmd.as_str());

                // Colour pad selected
                let out_message_enable: [u8; 11] =
                    [240, 0, 32, 41, 2, 12, 3, 0, ctl, SELECTEDCOLOUR, 247];
                match midi_comm.send(&out_message_enable) {
                    Ok(()) => (), //eprintln!("Sent message: {:?}", &out_message_enable),
                    Err(err) => eprintln!("Failed send: {:?}", err),
                };
            }
            None => (),
        };
        // eprintln!("run_ctl({}) finish", ctl);
    }
}

#[derive(Debug)]
struct LpxControl {
    // The source of truth for the state of the controls.  Includes the selected pad if there is one
    lpx_state: Arc<Mutex<LPXState>>,

    // The counter the monitoring thread uses.  When put to sleep
    // `sleeping` is set to this plus SLEEPDURATION
    counter: Arc<Mutex<usize>>,

    // Used to put controls to sleep when a MIDI key pressed
    // lpx_midi_sink: MIDICommunicator<()>,
    sleeping: Arc<Mutex<usize>>,

    // Change the colours of the LPX to reflect enabled/disabled
    // state.  Has to be called from the main thread of this class
    // that controls how long the LPX Ctl pads are inactive after
    // notes sent.  It is also used to change the colour of the
    // selected key
    lpx_midi: Arc<Mutex<MIDICommunicator<()>>>,
}
impl LpxControl {
    fn new() -> LpxControl {
        LpxControl {
            sleeping: Arc::new(Mutex::new(0)),
            lpx_state: Arc::new(Mutex::new(LPXState::new())),

            // For controlling the colours of the control pads
            lpx_midi: Arc::new(Mutex::new(
                MIDICommunicator::new(
                    "Launchpad X:Launchpad X MIDI 1",
                    "120-Proof-CTL",
                    move |_, _, _| {},
                    (),
                    2,
                )
                .unwrap(),
            )),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Return if the LPX control buttuns are "sleeping"
    fn sleeping(&self) -> bool {
        let g = self.sleeping.lock().unwrap();
        *g != 0
    }

    /// Put the controls on the LPX to sleep
    fn sleep(&self, s: usize) {
        // eprint!("sleep({}) start", s);
        let mut g = self.sleeping.lock().unwrap();
        let c = self.counter.lock().unwrap();
        *g = *c + s * 10;

        let lpx_state = self.lpx_state.clone();
        let lpx_midi = self.lpx_midi.clone();

        let mut lpx_midi_mut = lpx_midi.lock().unwrap();
        let mut lpx_state_mut = lpx_state.lock().unwrap();
        enable_lpx(false, &mut lpx_midi_mut, &mut lpx_state_mut);
    }

    /// Starts the thread that monitors `sleeping`.  It maintains a
    /// local counter `c`, wakes up periodically and increments `c`.
    /// It then compares the value of `c` to `sleeping`.  If `c >
    /// sleeping` then both `c` and `sleeping` reset to 0 and lpx is
    /// enabled.  Otherwise the lpx is dissabled.
    fn start(&self) {
        // eprintln!("start called");
        let sleeping = self.sleeping.clone();
        let lpx_state = self.lpx_state.clone();
        let lpx_midi = self.lpx_midi.clone();
        let counter = self.counter.clone();
        // Initialise
        {
            let mut lpx_midi_mut = lpx_midi.lock().unwrap();
            let mut lpx_state_mut = lpx_state.lock().unwrap();
            lpx_state_mut.active = false; // Force an update
            lpx_state_mut.last_pad = None; // No pad selected
            enable_lpx(true, &mut lpx_midi_mut, &mut lpx_state_mut);
        }

        thread::spawn(move || loop {
            // Sleep for 100 milli seconds
            thread::sleep(Duration::new(0, 100_000_000));
            {
                // eprintln!("Main sleep monitor loop top");
                let mut c = counter.lock().unwrap();
                *c += 1;
                let mut sleeping_mut = sleeping.lock().unwrap();

                // Check if we need to wake up
                if *c >= *sleeping_mut {
                    *sleeping_mut = 0;
                    *c = 0;
                    let mut lpx_midi_mut = lpx_midi.lock().unwrap();
                    let mut lpx_state_mut = lpx_state.lock().unwrap();
                    enable_lpx(true, &mut lpx_midi_mut, &mut lpx_state_mut);
                }

                // Check the locking buttons
                // match
                // eprintln!("Main sleep monitor loop bottom");
            }
        });
    }
}

#[derive(Debug)]
struct MidiCommTools {
    dispatcher: Dispatcher,
    lpx_control: LpxControl,
    locking_state: LockingState,
}
impl MidiCommTools {
    fn new() -> Self {
        let dispatcher = Dispatcher::new();
        let lpx_control = LpxControl::new();
        lpx_control.start();
        Self {
            lpx_control: lpx_control,
            dispatcher: dispatcher,
            locking_state: LockingState::Unlocked,
        }
    }
}

/// Allow the pad to be locked
#[derive(Debug, Clone, Copy, PartialEq)]
enum LockingState {
    Locked,
    Unlocked,
    Locking,
    Unlocking,
}

/// Maintiain information about the state of the LPX Two important
/// things: (1) Whether the control pads are active.  (2) The control
/// pad that was pressed last.  When the user uses the one (or more)
/// of the 64 MIDI pads in the main 8x8 grid deactivate the controls
/// so they cannot accidentally be pressed and unexpectedly change the
/// state of the instrument.  Remebering the active key is crucial so
/// when a control pad is pressed the cleanup script for the
/// previously activated key can be run
#[derive(Debug)]
struct LPXState {
    last_pad: Option<u8>,
    active: bool,
    locking_state: LockingState,
}
impl LPXState {
    fn new() -> LPXState {
        LPXState {
            last_pad: None,
            active: false,
            locking_state: LockingState::Unlocked,
        }
    }
}

/// Process a MIDI message
fn process_message(
    message: &[u8; 3],
    dispatcher: &mut Dispatcher, // defines which external programmes to run
    lpx_midi: Arc<Mutex<MIDICommunicator<()>>>,
    lpx_state: Arc<Mutex<LPXState>>,
) {
    // eprintln!("process_message({:?})", message);
    if message[0] == 176 {
        // A ctl message
        // eprintln!("process_message message({:?})", message);

        let pad = message[1];
        let vel = message[2];
        if pad >= 19 && vel > 0 {
            // There is some noise coming from the LPX with ctl-key 7
            // The rest are control signals that we want
            // The locked state of the LPX must be considered here.  Lock
            // the mode using pads: 91, 92, 93, 94 in succession and
            // unlock with 94, 93, 92, 91.  If locked reject any control
            // key but 94 (which starts unlocking).  If locking/unlocking
            // the pad must be the next in the sequence r state is swiched
            // to unlocked/locked.

            let lps = &mut lpx_state.lock().unwrap();
            eprintln!("state({:?}) message({:?})", lps, message);
            match lps.locking_state {
                LockingState::Locked => {
                    if pad == 94 {
                        lps.locking_state = LockingState::Unlocking;
                        return;
                    }
                }

                LockingState::Unlocked => {
                    if pad == 91 {
                        lps.locking_state = LockingState::Locking;
                        return;
                    }
                }
                LockingState::Locking => {
                    eprintln!("last_pad({:?}) pad({})", lps.last_pad, pad);
                    if lps.last_pad != Some(pad - 1) {
                        lps.locking_state = LockingState::Unlocked;
                        return;
                    } else if pad == 94 {
                        lps.locking_state = LockingState::Locked;
                        eprintln!("Locking");
                        return;
                    }
                }
                LockingState::Unlocking => {
                    if lps.last_pad != Some(pad + 1) {
                        // A pad other than 91, 92, 93, or 94 in sequence
                        lps.locking_state = LockingState::Locked;
                        return;
                    } else if pad == 91 {
                        lps.locking_state = LockingState::Unlocked;
                        return;
                    }
                }
            };
            // eprintln!("lps.locking_state({:?}) after block", lps.locking_state);

            // `dispatcher` will decide if any programmes get run
            lps.last_pad = Some(pad);
            if lps.locking_state != LockingState::Locked {
                // eprintln!("lps.locking_state({:?})", lps.locking_state);
                if pad < 91 {
                    // Do not run for locking pads
                    dispatcher.run_ctl(pad, lpx_midi);
                }
            }

            lps.last_pad = Some(pad);
        }
    }
}

/// Change the colour of the control pads.  Depending on the parameter
/// `enable`.  If `enable` is true the pads are being enabled and are
/// coloured green (87) and if !enabled the pads are being disabled
/// and are coloured red (5)
fn enable_lpx(enable: bool, lpx_midi: &mut MIDICommunicator<()>, lpx_state: &mut LPXState) {
    if lpx_state.active != enable {
        // eprintln!(
        //     "enable_lpx: enable({}) active_pad: {:?}",
        //     enable, lpx_state.last_pad
        // );
        let pad_colour = if enable {
            ENABLEDCOLOUR
        } else {
            DISABLEDCOLOUR
        };

        let active_pad: Option<u8> = lpx_state.last_pad;

        for i in 1..9 {
            let p = i * 10 + 9; // Pad
            if let Some(pad) = active_pad {
                if pad == p {
                    // Ignore the active pad
                    // eprintln!("Ignore pad: {}", &p);
                    continue;
                }
            }
            let out_message_colour_change: [u8; 11] =
                [240, 0, 32, 41, 2, 12, 3, 0, p, pad_colour, 247];
            match lpx_midi.send(&out_message_colour_change) {
                Ok(()) => (), //eprintln!("Sent message: {:?}", &out_message_colour_change),
                Err(err) => eprintln!("Failed send: {:?}", err),
            };
        }
        lpx_state.active = enable;
    }
}

/// Main loop.
/// Listen to the LPX MIDI and if it is a CTL signal process it, and
/// perhaps run some external programmes
fn run() -> Result<(), Box<dyn Error>> {
    // `midi_comm_tools` handles all communications with the LPX.  It
    // holds a `Dispatcher` and a `LpxControl`.  The `Dispatcher`
    // translates control messages from the LPX into actions on the
    // computer.
    // The `LpxControl` holds the data needed to sleep for a defined
    // period (a wake up and check the time model), A`LPXState` that
    // has the enabled/disabled state as well as the active LPX pad,
    // and a `MIDICommunicator` to change the pad colours on the LPX
    let midi_comm_tools = MidiCommTools::new();

    // The main loop is the closure in this communicator
    let _foo = MIDICommunicator::new(
        "Launchpad X:Launchpad X MIDI 2",
        "120-Proof-CTL",
        move |_stamp, message, midi_comm_tools| {
            // eprintln!(
            //     "{}: Msg: {:?} (len = {})",
            //     (stamp as f64) / 1_000_000.0,
            //     &message,
            //     message.len()
            // );

            // The messages that wil be processed here are length
            // three.  MIDI notes are also length three, and when they
            // come by the controls are inactivated for a period to
            // avoid accedentally changing the set up of the
            // instrument
            if message.len() == 3 {
                if message[0] == 176 {
                    if !midi_comm_tools.lpx_control.sleeping() {
                        let array = <[u8; 3]>::try_from(message).unwrap();
                        process_message(
                            &array,
                            &mut midi_comm_tools.dispatcher,
                            midi_comm_tools.lpx_control.lpx_midi.clone(),
                            midi_comm_tools.lpx_control.lpx_state.clone(),
                        );
                    }
                } else if message[0] == 144 {
                    // A MIDI note
                    if midi_comm_tools.locking_state != LockingState::Locked {
                        // No point in going to sleep if locked
                        midi_comm_tools.lpx_control.sleep(SLEEPDURATION);
                    }
                }
            }
        },
        midi_comm_tools,
        1,
    )?;

    loop {
        // Infinite loop
        thread::sleep(Duration::new(1, 0));
    }
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {}", err),
    }
}
