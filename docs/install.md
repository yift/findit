# Installing findit
There are a few ways to install `findit`:

## From source
To install `findit` from source, make sure you have the Rust toolchain installed. See details [here](https://www.rust-lang.org/tools/install)


### Using Cargo
To install `findit` using Cargo, you can simply run:
```bash
cargo install findit
```


### From the repository
To install `findit` from the repository, you need to clone the repository, build the tool, and copy the executable to your path. For example (assuming ~/bin is in your path):
```bash
git clone https://github.com/yift/findit
cd findit
cargo build -r
cp target/release/findit ~/bin
```

## From Docker
You can use the `findit` Docker container. Please note that this will allow you to access only the files in the container volume. For example:
```bash
docker run -it --rm -v $(pwd):/data yiftach/findit -m /data
```
(To install Docker, see [here](https://docs.docker.com/engine/install/)).

## From binary
Some operating system binaries are available in the [latest release](https://github.com/yift/findit/releases/latest)

