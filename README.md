# tenv (temporary environment)
Small CLI app for running commands with temporary environment variables. Super useful on Windows/Powershell where you can't easily set temp variables inline with command being run.
## Installation
Install from http://crates.io via `cargo`
```
cargo install --locked tenv
```
Install from source
```
cargo install --path .
```

## Arg Files
Supports `argfile`s. see [example.tenv](./example.tenv) for an example. Can be used to set up an environment across multiple projects. For instance here's one that sets up a flutter environment:
```
# Flutter
-p C:\dev\flutter\bin
# Android SDK Locations
-e ANDROID_SDK_ROOT=C:\Users\user1\AppData\Local\Android\Sdk
-e ANDROID_NDK_ROOT=$ANDROID_SDK_ROOT\ndk\25.0.8775105
# Android SDK binaries
-p $ANDROID_SDK_ROOT\tools\bin
-p $ANDROID_SDK_ROOT\emulator
```

## Uses
### Running commands
Bash
```bash
# Add ~/hugo to PATH and run `hugo` program
tenv -p ~/hugo hugo
```
Powershell
```bash
# Add C:\dev\hugo to PATH and run `hugo` program
tenv -p C:\dev\hugo hugo
```

Especially useful for RUST_BACKTRACE on Powershell
```powershell
tenv -e RUST_BACKTRACE=1 "cargo run"
```
### Shell environment
Bash
```bash
# spawn new bash shell with env vars APPLES="Red" and BANANAS="Yellow"
tenv -e APPLES=Red -e BANANAS=Yellow bash
```
Powershell
```powershell
<# spawn new Powershell with env vars APPLES="Red" and BANANAS="Yellow" #>
tenv -e APPLES=Red -a BANANAS=Yellow powershell
```

## Caveat
Because some shell commands aren't directly callable from Command::new() (i.e. the `cat` alias on Powershell), tenv runs programs directly through powershell on Windows and bash otherwise.