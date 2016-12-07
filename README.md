speedtest (stest)
=================


This is a port of https://github.com/sivel/speedtest-cli into rust, as I need to have no runtime dependencies. And I'm not planning to support all the arguments used in original tool.
I'd like to add support for most platforms as a single stand alone binary that could work out of the box.

![Alt Text](https://github.com/opensourcegeek/stest/raw/master/stest-i686.gif)

Usage
-----

Supported options given below. Argument parsing is powered by awesome https://github.com/kbknapp/clap-rs

```
USAGE:
    stest [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --csv <csv>                          Set name of csv file
    -n, --number-tests <number_tests>        Sets number of tests to run
    -s, --server-country <server_country>    This will scan servers only from given country - it might take a while before it finds the best server
```


to-do
-----

   - Add command line switches to use specific server, override geo-ip lookup.
   - Add command line argument to run in server mode (no security) again rendering CSV/JSON file. I'm thinking websocket but not sure!
