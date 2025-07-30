# Rust Synthesizer with GUI & Audio Effects

Software synthesizer programmed in Rust. Built using [rodio](https://crates.io/crates/rodio) for audio playback, [minifb](https://crates.io/crates/minifb) for user input handling, and [image](https://crates.io/crates/image) for sprite rendering. All visual assets are made by me, Hichael, using [Aseprite](https://www.aseprite.org/).

## Features

**Multiple Waveforms**: Sine, Square, Triangle, and Sawtooth waves

**ADSR Envelope**: Full Attack, Decay, Sustain, Release control

**Real-time Audio Effects**: Delay, Reverb, and Flanger

**Recording & Playback**: Record and play them performances in loops

**Interactive GUI**: Mouse and keyboard controls for all parameters


## Requirements
* [Rust](https://www.rust-lang.org/tools/install)

## Cargo dependencies

* [rodio](https://crates.io/crates/rodio)
* [minifb](https://crates.io/crates/minifb)
* [image](https://crates.io/crates/image)

## Running program: Cargo

The shell script 'up' builds and runs our application by executing the following:
```
1. cargo build
2. cargo run
```

## Running program: x86 executable for Windows

One may also run an executable directly. This has been compiled for target 'x86_64-pc-windows-msvc'
utilizing 'cargo build --release'
```
./synthesizer.exe
```

## Screenshot
![screenshot](rust_synthesizer_screenshot.png)

## Synthesizer Key Controls
Musical Notes:

    Q: Play musical note C in octave 4 (261.63 Hz)
    2: Play musical note C# in octave 4 (277.18 Hz)
    W: Play musical note D in octave 4 (293.66 Hz)
    3: Play musical note D# in octave 4 (311.13 Hz)
    E: Play musical note E in octave 4 (329.63 Hz)
    R: Play musical note F in octave 4 (349.23 Hz)
    5: Play musical note F# in octave 4 (369.99 Hz)
    T: Play musical note G in octave 4 (392.00 Hz)
    6: Play musical note G# in octave 4 (415.30 Hz)
    Y: Play musical note A in octave 4 (440.00 Hz)
    7: Play musical note A# in octave 4 (466.16 Hz)
    U: Play musical note B in octave 4 (493.88 Hz)

Octave Control:

    F1: Decrease the octave (0 is minimum)
    F2: Increase the octave (6 is maximum)

Waveform Control:

    S: Toggle waveform between sine, square, triangle, and sawtooth

ADSR Envelope Control:

    F3: Decrease Attack (0-99, controls fade-in time)
    F4: Increase Attack
    F5: Decrease Decay (0-99, controls fade from peak to sustain)
    F6: Increase Decay  
    F7: Decrease Sustain (0-99, controls held volume level)
    F8: Increase Sustain
    F9: Decrease Release (0-99, controls fade-out time)
    0:  Increase Release

Audio Effects Control:

    F10: Toggle Delay Effect (250ms delay with feedback)
    F11: Toggle Reverb Effect (Schroeder reverb algorithm)
    F12: Toggle Flanger Effect (Modulated delay with LFO)

Recording & Playback:

    Mouse: Click REC button to start/stop recording
    Mouse: Click PLAY button to playback recorded notes (loops automatically)
    Mouse: Click STOP button to halt all audio and recording

## Mouse Controls



**Piano Keys**: Click white keys (C, D, E, F, G, A, B) to play notes

**Sharp Keys**: Click black keys (C#, D#, F#, G#, A#) for sharp notes  

**Waveform Display**: Click to cycle through waveforms (Sine → Square → Triangle → Sawtooth)

**Octave Fader**: Click upper half to increase octave, lower half to decrease

**ADSR Faders**: Click and drag the Attack, Decay, Sustain, Release faders

**Effects Buttons**: Click DLY, REV, FLG buttons to toggle audio effects

**Control Buttons**: Click REC, PLAY, STOP for recording functionality

