speedtest (stest)
=================

[![Build Status](https://travis-ci.org/opensourcegeek/stest.svg?branch=master)](https://travis-ci.org/opensourcegeek/stest) [![Build status](https://ci.appveyor.com/api/projects/status/e86ogv74rvs70529?svg=true)](https://ci.appveyor.com/project/opensourcegeek/stest)


This is a port of [speedtest-cli](https://github.com/sivel/speedtest-cli) into rust, to avoid having runtime dependencies. Currently not all the arguments are supported. I'd like to add support for most platforms but targeting only linux for now. Ideally it will be a single stand alone binary that will work out of the box.



Download
--------

Download the binaries 'stest-i686-linux' for 32-bit or 'stest-x86_64-linux' for 64-bit linux from [releases](https://github.com/opensourcegeek/stest/releases) for most recent version. These are the only supported platforms for the moment.


Usage
-----

Supported options given below. Argument parsing is powered by awesome [clap-rs](https://github.com/kbknapp/clap-rs)

```
USAGE:
    stest [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -u, --use-cached    Use a cached copy of servers
    -V, --version       Prints version information

OPTIONS:
    -c, --csv <csv>                                    Set name of csv file
    -n, --number-tests <number_tests>                  Sets number of tests to run
    -s, --server-country <server_country>
            This will scan servers only from given country name - it might take a while before it
            finds the best server
    -o, --server-country-code <server_country_code>
            This will scan servers only from given country code - it might take a while before it
            finds the best server

```


to-do
-----

   - Add command line switches to use specific server, override geo-ip lookup.
   - Add command line argument to run in server mode (no security) again rendering CSV/JSON file. I'm thinking websocket but not sure!
