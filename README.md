## llama

### the Low Level ARM Machine Amulator (of course)

---

### What is llama?

Llama is an experimental emulator for the Nintendo 3DS's ARM9 (with a very limited ARM11 implementation as well).

Llama most certainly cannot run any 3DS games. Don't even try.

### What does it look like?

See for yourself!

![Llama's GUI, running Hourglass9](https://i.imgur.com/dl5YOH1.png)

> Source-level debugging in a GDB TUI? That's a lot of buzzwords!

### How do I use it?

First, you have to build llama from source. See below.

#### Loading applications

Llama loads binaries from either a [FIRM](https://www.3dbrew.org/wiki/FIRM) file or a "ctr9" package. 

A ctr9 package is a directory named `[dirname].ctr9`, with the following structure:

```
foo.ctr9:
|- desc.json
|- ...
```

#### "desc.json"

The `desc.json` file describes how llama will load your ARM9 binaries. `desc.json` files look like this:

```
{
    "entryPoint": "0x0801B01C",
    "entryPoint11": "0x1FFAD034"
    "binFiles": [
        { "bin": "firm_0_1FF00000.bin", "vAddr": "0x1FF00000" },
        { "bin": "firm_1_1FF80000.bin", "vAddr": "0x1FF80000" },
        { "bin": "firm_2_08006800.bin", "vAddr": "0x08006800" }
    ],
}
```

- `entryPoint`: Address at which llama will begin executing the ARM9 processor.
- `entryPoint11`: Address at which llama will begin executing the ARM11 processor.
- `binFiles`, `binFiles11`: Array of binaries found within the ctr9 package.
  - `bin`: The binary filename.
  - `vAddr`: Address where llama will copy the binary.

#### Debugger

Llama will not automatically begin running the ctr9 package upon opening. To run, press the play/pause button or use the `run` debugger command.

Llama has a semi-useful built-in debugger controlled with textual commands.

- `run`: Unpauses the loaded program.
- `cpu <arm9|arm11>`: Switches between actively debugged CPUs
- `asm [address hex]`: Prints disassembly for the current instruction.
- `brk <address hex>`: Adds a CPU breakpoint at the specified address.
- `btn [button] [up/down]`: Toggles a button or prints full button state.
- `irq <type>`: Triggers an interrupt request of the specified type.
- `keydmp`: Dump AES keys.
- `mem <start address hex> [# bytes hex] [dumpfile.bin]`: Prints n bytes of memory from the specified address, optionally dumping to file.
- `reg [register name]`: Prints specified register, or all registers if none specified.
- `step`: Runs one CPU instruction.

### What can I use it with?

My [crossbar9](https://github.com/archshift/crossbar9) repository can be used as a template for Rust programs that should run on both llama and the actual 3DS.

### How do I build it?

Llama is written in Rust and C++, which means you need a compiler for both languages installed on your system.

The GUI uses Qt5, which must be installed as well. Make sure you also have QtQuick/Qt-declarative.

#### Miscellaneous dependencies:

- Capstone disassembler

#### Actually building

Once all dependencies are installed, building should be as easy as running:

```
cargo build --release
```

### License

Llama is licensed under the BSD 3-clause license.

Included icons were created by [material.io](material.io) and are licensed under Apache v2.0.
