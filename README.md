# rdm
A configuration manager written in rust and configurable in lua.

## Usage
rdm uses git to manage your configuration. So you first need to create a bare
repository. This can be done with:
```shell
rdm init # --help to check the different options of this command.
```

Then to add some files to your configuration repository, you can do
```shell
rdm add <path>
```
or add to your rdm config file (default is `.config/rdm/init.lua`)
```lua
file("<path>")
-- and for a full directory
directory("<path>")
```

## Future features
- [ ] Clone a configuration
- [ ] Remote management
- [ ] Copy files
- [ ] Handle multiple hosts
