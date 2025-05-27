# embassy-litex
embassy-rs hardware abstraction layer for LiteX / vexriscv SoC


As this is my first project in Rust, please consider this library as unstable and untested. 

## Run the example

An example is provided which can run in a simulated SoC. LiteX with all requirements is needed.

#### LiteX installation guide
See: https://github.com/enjoy-digital/litex/wiki/Installation
1. Install Python 3.6+ and FPGA vendor's development tools and/or Verilator.
2. Install Migen/LiteX and the LiteX's cores:
```shell
$ wget https://raw.githubusercontent.com/enjoy-digital/litex/master/litex_setup.py
$ chmod +x litex_setup.py
$ ./litex_setup.py --init --install --user     #(--user to install to user directory)
```

#### Initialize submodules
```shell
$ git submodule init
```

#### Build the example app
```shell
$ cd ./example
$ ./build.sh
```

#### Run simulation
```shell
$ ./run_sim.sh
```




