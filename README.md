# Reburn

## The idea

Each time a change is detected on the filesystem the building/launching script is run again

A cool way to do so could be placing `reburn` as the shebang interpreter:
```sh
#!reburn /src/**/*.rs
cargo run
```

When a change is detected, the current instance is killed if it was alive and the script is launched again

## The problem

If the script is run using reburn we do not know the right interpreter to launch the script

The shebang does not force the data next to the interpreter to be split as arguments so we should not rely on a syntax like:
```sh
#!reburn -i bash /src/**/*.rs
```

Although it would be could, the arguments would be
```rs
["reburn", "-i bash /src/**/*.rs", "file.sh"]
```

Relying on the extension or any other building file naming tricks would not be cool either

- file.sh => .sh => bash => :(
- file_bash => _bash => bash => :(

## The solution

A second shebang
```sh
#!reburn /src/**/*.rs
#!bash
```
Why? A second shebang would be easy to parse, also it feels natural to wrap an already working script with the `reburn` shebang in order to provide the reloading feature