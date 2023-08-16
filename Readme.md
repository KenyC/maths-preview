Maths Preview
=======================

A fast and minimal WYSIWYG for LateX mathematical formulas.

<p align="center"><img src="screenshots/demo.gif" alt="Demo GIF" width="300px"/></p>

Features:

  - **Real-time rendering:** formula is updated instantly as you type.
  - **SVG export:** formula renders can be saved as svg.
  - **Desktop and web version:** try [it](https://maths-preview.netlify.app/) online or see below for desktop version.
  - **Plug-ins:** integrates with [Sublime Text](#sublime-text) and [LibreOffice Writer](#libreoffice-writer).



## Overview

### Usage

 - Build and launch the program (from Cargo: `cargo r`), 
 - Type a formula in text input
 - See it update in the display in real-time. 
 - Close the app ; some output is generated either on stdout or written to a file. 


### Example

Supposing you have the executable on your `$PATH` as `maths_preview`:

```bash
maths_preview -i "e^{i\pi}+1=0" -f svg -o /tmp/out.svg
```

 - `-f svg` specifies can be used to output a SVG render upon exit, `-f tex` just outputs the formula you typed in. 
 - `-o` specifies where the output file will be written. If left unspecified, the output will be provided on stdout. 
 - `-i` specifies which formula is displayed on program start.

### Full description of the options

```
Application Options:
  -m, --mathfont             Path to an OpenType maths font to use for render (default: STIX Maths, bundled in the executable)
  -i, --informula            Formula to edit (default: $\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}$)
  -o, --outfile              Output file ; if left unspecified, output is directed to stdout.
  -d, --metainfo             Whether to output meta-info on stdout (baseline position, font size, formula, etc.). If 'outfile' is not specified, stdout will contain both the output and the meta-info
  -f, --format               Format of 'outfile' ('svg', 'tex') ; defaults to 'tex'
  -s, --fontsize             Size of font in the SVG output (default: 10)
  --display=DISPLAY          X display to use
```

## Building & installing

Run:

```bash
cargo b --release
```

The program depends on some Rust crates and the GTK3 library. Cargo will take care of the Rust dependencies but you will need to install the GTK3 library and its development files yourself. 

The steps to install the development files needed for GTK3 depend on the OS and distribution. 

  - Debian/Ubuntu (apt): `sudo apt install libgtk-3-dev`.
  - Fedora/RHEL (dnf): `sudo dnf install gtk3-devel`
  - Mac (brew): `brew install gtk+3`
  - Windows: I'm unsure as of yet, [this guide](https://www.gtk.org/docs/installations/windows/) may help

When build is complete, the executable should be under `target/release/maths_preview`. You can add it to your PATH, e.g. by copying it to in `~/bin` (Linux).

## Plugins

### Sublime Text

<p align="center"><img src="screenshots/sublime_text_demo.gif" alt="Demo Sublime Text plugin"/></p>

Under `clients/sublime-text/`, you will find a Sublime Text package providing the command `MathsPreview: Insert Formula` to use Maths Preview in Sublime Text.

#### Installation


  - copy the package folder in your Package folder (which you can find with the command `Preferences: Browse Packages`)
  - specify the path to the Maths Preview executable in the `math_preview_exe_path` field of the `InsertFormula.sublime-settings` file.

#### Usage

  - Launch `MathsPreview: Insert Formula` at any position in the document (accessible from command palette) ; the executable pops up.
  - Write the formula and exit the window, by using e.g. Esc.
  - The formula you typed is inserted in the document.


### LibreOffice Writer

<p align="center"><img src="screenshots/libreoffice_writer_demo.gif" alt="Demo LibreOffice Writer plugin"/></p>

#### Installation

Under `clients/libreoffice-writer`, run `make`. This will zip the add-on and automatically add it to LibreOffice Writer.

#### Usage

  - Click on `Inline Formula` or `Block Formula` in the toolbar.
  - Write the formula and exit the window, by using e.g. Esc.
  - The formula you typed is inserted in the document. 

#### Adding keyboard shortcuts (aka accelerators)

  - Go to  `Tools > Customize ...`. 
  - Click on the `Keyboard` tab. 
  - You'll see a list of keys at the top and a list of functions at the bottom. In the `Category` list at the bottom, navigate to `LibreOffice Macros > My Macros > InsertFromMathsPreview.oxt > insert_formula`.
  - A list of functions appears in the bottom middle panel. The two relevant funtions are `insert_inline_formula` and `insert_block_formula`. Click on the one you want to assign a keyboard shortcut to.
  - Click on the key combination you want the function assigned to in the upper panel.
  - In the right panel, click on `Modify`.


<p align="center"><img src="screenshots/assign_keyboardshortcuts_in_libreoffice_write.png" alt="Screenshot illustrating how to assign keyboard shortcuts in LibreOffice Writer" width="50%"/></p>



