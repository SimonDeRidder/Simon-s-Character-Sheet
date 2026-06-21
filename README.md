# Simon's Character Sheet

This repository contains a webapplication to host an interactive character sheet for D&D 5e (2014 rules).
The front end is HTML+CSS, while the "back end" (run by the browser) is JavaScript and WebAssembly (from Rust).
This back end is predominantly based on the document-level JavaScript that is used in [MPMB's Character Record Sheet](https://github.com/morepurplemorebetter/MPMBs-Character-Record-Sheet).

## How to use

### Download

You can download the latest release from the [releases page](https://github.com/SimonDeRidder/Simon-s-Character-Sheet/releases).
Simply download and extract the `.tar.gz` or `.zip` to a folder of you choice.
Alternatively, you can [build the Webassembly from source yourself](#building-from-source).

### Run

Simply opening `index.html` will cause CORS issues in most browsers.
You will have to run a server to serve the files to your browser.
Fortunately, this is quite simple, and can be done in multiple ways.
Here are some of them:

#### Running with Static Web Server (SWS)

This is a blazing fast file server built in Rust.
You can download it [here](https://static-web-server.net/download-and-install).
Then, in the downloaded folder, run
```sh
static-web-server --port 80 --root .
```
Then, you're ready to [Open the character sheet in your browser](#open-the-character-sheet-in-your-browser).

#### Running with Python
A very easy (though not quite so secure) way to set up a simple server with Python is to use the built-in python module (inside the downloaded folder):
```sh
python -m http.server 80
```
You can replace 80 with whatever port you like. On Windows, see [here](https://www.wikihow.com/Install-Python-on-Windows) to install Python.

Then, you're ready to [Open the character sheet in your browser](#open-the-character-sheet-in-your-browser).

#### Running with Node.js

To install Node.js, see [here](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs).
With Node.js installed, install a http server with
```sh
npm install http-server -g
```
Then, in the downloaded folder, simply run
```sh
http-server
```
Then, you're ready to [Open the character sheet in your browser](#open-the-character-sheet-in-your-browser).

#### Open the character sheet in your browser

With the server running, you can open the sheet with the following url on your own device:
- http://localhost:80

(If you changed the port in the server command, also change it in this url. If you use port 80, you can omit the ":80", it is the default.)

For other devices on a local network, [find your local ip](https://www.wikihow.com/Find-an-IP-Address), and use
- `http://<local_ip>:80`

To share over the public internet, make sure to [forward the port](https://www.wikihow.com/Set-Up-Port-Forwarding-on-a-Router) in your router, and find out your public ip address (e.g. at https://www.whatismyip.com).
The url for your friends is then:
- `http://<public_ip_address>:<forwarded_port>`


### Building from source

This is not necessary when you've downloaded a pre-built binary from the [releases page](https://github.com/SimonDeRidder/Simon-s-Character-Sheet/releases), but if you like, you may build the WebAssembly from its Rust source code.

1) Install the Rust ecosystem. The easiest way to do this is with [rustup](https://rustup.rs).
2) Set WASM as a target with
	```sh
	rustup target add wasm32-unknown-unknown
	```
2) Install wasm-pack with
	```sh
	cargo install wasm-pack
	```
	and wasm-bindgen-cli with
	```sh
	cargo install wasm-bindgen-cli@0.2.108
	```
3) Build the WASM folder with the included `build_release.sh`. Alternatively, run wasm-pack directly with
	```sh
	wasm-pack build -m no-install --no-typescript -t web --release -d wasm --out-name wasm --no-pack
	```

Now you're ready to go ahead and [Run](#run).

## Extra content

This web application works with extra content written for MPMB's Character Record Sheet. You can find such additional content at the [/r/mpmb subreddit](https://www.reddit.com/r/mpmb/). There is also [the MPMB Discord sever](https://discord.gg/Qjq9Z5Q) for discussion and questions.

For syntax to write your own, consult [additional content syntax](additional%20content%20syntax).

Put the files in the `additional_content`` folder and import them into the sheet using the "Import" button.
Each file is a complete script. You can add multiple files, but take note that they will be processed in the order they are added.
Please see the [How-To Guide - Add More Content](https://www.flapkan.com/how-to/add-more-content) on my website for an explanation how to add these scripts to the PDF.

## Known issues

- the global buttons are ugly and only in the upper left corner of the stats tab
- some of the context menus are not scrollable

## Legal Information
Simon's Character Sheet automates some of the administrative tasks around playing the game of Dungeon & Dragons 5th edition &copy; Wizards of the Coast, Inc.

The `_functions`, `_variables`, `additional content` and `additional content syntax` are under Copyright &copy; 2014 Joost Wijnen; Flapkan Productions

The files in these folders are modified to integrate with the rest of the application.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU General Public License for more details.

You can find a copy of the GNU General Public License along with this program.
If not, see <http://www.gnu.org/licenses/>.

This work includes material taken from the System Reference Document 5.1 (“SRD 5.1”) by Wizards of the Coast LLC and available at https://dnd.wizards.com/resources/systems-reference-document. The SRD 5.1 is licensed under the Creative Commons Attribution 4.0 International License available at https://creativecommons.org/licenses/by/4.0/legalcode.
