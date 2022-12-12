# Reburn

Automatic script restarter on file change detection

## How does it work?

### Terminal invocation
Example for cargo project, will reload when any file under `src` folder changes
```sh
$ reburn "src/**" -- cargo run
```

### Embedded in a script
Create a python script that runs whenever itself changes
```py
#!reburn file.py
#!python
print("Running...")
```
Can be launched as a normal script
```sh
$ ./file.py
```
Why a second shebang? It's easy to parse, also it feels natural to wrap an already working script with the `reburn` shebang in order to provide the reloading feature