gameserver
==========

The gameserver exists as 2 separate executables, a **ticker** that periodically pings vulnboxes, and a **web** server that listens for requests to submit flags(and also displays a scoreboard).

Ticker Configuration
--------------------

Run the `ticker` executable with the `--config` flag to specify a configuration file. This file should be in the `toml` format and should contain these fields:

```toml
db = "test.db"
services_dir = "../services"
bind_addr = "127.0.0.1:3300"
secret_key = "secret_key"

flag_period = 10
check_period = 5
delay = 3
timeout = 15

ignores = ["service1", "service2"]

teams = [
    { id = 1, ip = "127.0.0.1" },
]
```

Database
--------

No migrations will be made, since no schema changes will be made while the event is running. Instead, run `init.sql` on the target sqlite database before the game begins.

Flag Format
-----------

The flag format should be treated by the services as an opaque string in a format matching the regular expression: `flag\{[0-9A-Fa-f]{64}\}`.

Service API
-----------

The name of the service is determined by the name of the directory containing its source files. A configuration file is required and must be found at `meta.toml` in the service's root directory. Here is an example:

```toml
port = 9999

atk_score = 50
def_score = 50
up_score = 50

get_flag = "<path to get_flag executable>"
check_up = "<path to check_up executable>"
set_flag = "<path to set_flag executable>"
```

The paths listed under `client` are optional; if they are not provided, the gameserver will automatically look for the `get_flag`, `check_up`, and `set_flag` executables. Otherwise, they must be strings pointing executable files. Here are their usages:

```
Usage: ./get_flag [ip] [port] [flagid]
Returns: 0 on success + flag in stdout
```

```
Usage: ./check_up [ip] [port]
Returns: 0 on success
```

```
Usage: ./set_flag [ip] [port] [flag]
Returns: 0 on success + optional flagid in stdout
```

In all the previous examples, IP will be a string, like `"127.0.0.1"`, and port will be an integer.

Contact
-------

Author: Michael Zhang

License: MIT
