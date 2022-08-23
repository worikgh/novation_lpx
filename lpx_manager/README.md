# lpx_manager

Map keys from LPX onto midi notes 

Light up the pads according to the status of the generated note

Currently:

* red for the root
* green for other notes on the scale
* white for notes not on  the scale

As a pad is pressed it is coloured purple. 

All the notes (12 per octave) are mapped to pads.

The of notes in the three left columns are repeated on the three right
columns, a row down.

Equivalently the notes of the three right columns are repeated o the three left columns a row up.

## Building

In the root directory of this repository `novation_lpx/` run: `cargo build --release`

The executable is `target/release/lpx_manager` and can be copied to wherever it is needed)

## Running

### Arguments

1. Path to a configuration file for MIDI connections

2. The root note in MIDI.  60 is middle C

3. The scale.  This is defined as one to twelve integers in the range 1 - 12 inclusive, and ordered, that define the notes of the scale.  Always starts with `1`

* Example

	`./lpx_manager lpx.cfg 60 1 4 6 8 11` 


### MIDI configuration

There are three MIDI connections to the LPX

1. `midi_source_lpx` that outputs  the MIDI note signals from the LPX

2. `midi_sink_lpx` where the LPX receives signals to change pad colours

3. `midi_sink_synth` the midi connection to a synthesiser.

Example:

```
midi_source_lpx:Launchpad X:Launchpad X MIDI 2
midi_sink_lpx:Launchpad X:Launchpad X MIDI 1
midi_sink_synth:yoshimi-INSTANCE_03:input
```

`midi_source_lpx` and `midi_sink_lpx` will always be the same.

### Demo

In the `demo` directory is a Perl script to run `lpx_manager`.  It has all the files, including compiled binaries in that directory.

The MIDI connections must be edited by hand because `yoshimi` connects itself tto the LPX directly.  Use a MIDI editor (like [qjackctl](https://qjackctl.sourceforge.io/) to clean up the MIDI otherwise there will be two notes playing for each pad.
