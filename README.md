speedtest (stest)
=================


This is a port of https://github.com/sivel/speedtest-cli into rust, as I need to have no runtime dependencies. And I'm not planning to support all the arguments used in original tool.
I'd like to add support for most platforms as a single stand alone binary that could work out of the box.

![Alt Text](https://github.com/opensourcegeek/stest/raw/master/stest-i686.gif)

to-do
-----

   - Add command line argument to run N tests.
   - Add command line switches to use specific server, override geo-ip lookup.
   - Add command line switch to create CSV/JSON file output
   - Add command line argument to run in server mode (no security) again rendering CSV/JSON file. I'm thinking websocket but not sure!
