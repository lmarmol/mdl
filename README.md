# Momentos Downloader
 
MDL is a command line tool that allows you to list and download Momentos contents in bulk.
 
## Usage
 
The `mdl` tool has three subcommands: `login`, `list`, and `download`. You can learn more about them by running 

```sh
./mdl --help
```
 
### Logging into the Momentos
 
Before accessing Momentos contents, users must be logged into the system. This can be done using the `login` subcommand.
 
```sh
./mdl login --user email.address@server.com
```
 
Then you will be prompted to enter your password. After a successful login request, `mdl` will create a local file `mdl.toml` with your access token. Subsequent commands will read your access token from that file.
 
### Listing Momentos Contents
 
Use can list their Momentos contents by issuing the following command:
 
```sh
./mdl list
```
 
The output consists of a list of pairs `<coruse-id> <coruse-name>`. The `<course-id>` will be used to download the course contents.
 
### Downloading Momentos Contents

To download your contents, use the `download` subcommand and provide the `<course-id>` of the course you want to download.
 
```sh
./mdl download --id <course-id> --id <second-course-id>
```
 
`mdl` will then create a folder named after your `course-id` where it will place all the lectures and transcripts from such course.
 
 
## Building from source
 
```sh
cargo build --release
```
 
### Development dependencies
 
* Rust toolchain
* A C compiler and linker
* SSL dev library
 
The recomended way of installing the Rust toolchain is using `rustup`. You can find more information [here](https://rustup.rs/).
 
The rest of the dependecies can be installed, in Ubuntu, with the following command.
 
```sh
sudo apt install build-essential libssl-dev
```