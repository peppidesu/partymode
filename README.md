# partymode
Prevents your system from idling/suspending while media is being played.



## Dependencies
- `dbus`
- `systemd`

## Installation
### crates.io

```sh
cargo install partymode
```

## Usage
```
partymode [OPTIONS] <COMMAND>

Commands:
  daemon  Run the partymode daemon
  on      Enable party mode
  off     Disable party mode
  toggle  Toggle party mode
  status  Show the current status
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <PATH>  Provide a custom location for the config file
  -v, --verbose        Enable verbose logging
  -h, --help           Print help
  -V, --version        Print version
```

### Systemd unit
To manage the partymode daemon with systemd, you can create this user service:

```desktop
[Unit]
After=dbus-broker.service
Description="partymode - Keep your system awake while playing media"
Documentation="https://github.com/peppidesu/partymode"

[Service]
Type=simple
ExecStart=/path/to/partymode daemon # change this
Restart=on-failure

[Install]
WantedBy=default.target
```

## Configuration
`partymode` will create the following config at `~/.config/partymode/config.toml` (or the directory specified with `-c`) if it doesn't exist:

```toml
default-enabled = true
poll-interval = 5000

["*"]
always = false
mode = "block"
targets = ["idle"]
```

#### `default-enabled`
Whether to enable partymode on startup or not.

#### `poll-interval`
How often to check for player changes, in ms.

### Rules
Rules allow you to specify inhibit behavior on a per-application basis. A rule looks like this:

```toml
[name]
always = false
mode = "<mode>"
targets = []
```

#### `always`
When true, inhibit regardless of whether partymode is enabled or not.

#### `mode`
Inhibit mode as specified by [systemd-inhibit(1)](https://www.freedesktop.org/software/systemd/man/latest/systemd-inhibit.html#--mode=).

#### `targets`
What to inhibit. Can be one of `"idle"`, `"suspend"` or `"shutdown"`.

#### Rule names
`partymode` will use the last part of the MPRIS bus name (`org.mpris.MediaPlayer2.<name>`) to match against config rules. You can get a list of these with `playerctl`:
```sh
playerctl -l
```

#### Default rule
If parts of a rule are omitted or no matching rule is found, `partymode` resorts to using the default rule (`["*"]`).

## FAQ
### Wait, isn't this just what `<other project>` does?
`partymode` is different from most inhibit tools like `caffeine-ng` or the built-in KDE menu, because they don't automatically inhibit when media is playing. These tools aim to solve a different problem.

To give an example, `partymode` will allow you to inhibit suspend while listening to music, regardless of whether that window is fullscreen or not. When you stop playback, `partymode` will also stop inhibiting.

### Why does my system still suspend even if `partymode` is enabled?
`partymode` being enabled doesn't mean it will force inhibit, it only means it will apply the rules defined in the configuration.

## Contribute
Feel free to report issues and PR :)
