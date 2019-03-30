# Tomato timer built in rust designed to run in the command line
[![Build Status](https://travis-ci.org/louisdewar/tomato.svg?branch=master)](https://travis-ci.org/louisdewar/tomato)

## WIP: No where near polished!

This is based on [the pomodoro technique](https://en.wikipedia.org/wiki/Pomodoro_Technique), where you have work sessions then breaks.
The default is to have 25 minutes doing work then 5 minutes of break. You do this until you have completed 4 work sessions which is when you get a longer break (default 20 minutes).

It is currently possible to customise *some* settings although this is minimal. The best use right now is to have a script that can be run whenever either work or a break starts. This is not currently documented but it is possible to try to examine the source code to work out how to achieve this. Obviously in the future this will be documented, once the feature is properly built.

## Running

[Install rust](https://rustup.rs/).

Then: `cargo run --release`

## Testing

Currently there are no unit tests, since it is difficult to test timing. Rust allows a certain trust that if the code compiles it will mostly work. The code is also tested with `cargo clippy`, also travis will check the code is formatted according to rust format (it runs `cargo clippy`)
