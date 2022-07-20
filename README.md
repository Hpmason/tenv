# tenv (temporary environment)
Small cli app for running commands with temporary environment variables. Super useful on Windows/Powershell where you can't set temp variables right before running a command.
```
tenv apples=cool bananas=gross python
```

## Caveats
Because some shell commands aren't directly callable from Command::new() (ie cat on Powershell), tenv tried to run command regularly, then if command is not found it tried to run it via hardcoded shells (bash on Linux/MacOS and powershell on Windows).