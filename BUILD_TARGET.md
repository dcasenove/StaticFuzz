# Example: how to build Objdump

## 1) Build ParmeSan
Use the included script `build/build.sh` to build ParmeSan and the required tools.

We really recommend you install the LLVM version supplied by the `build/install_llvm.sh` script. At the end it will show the env vars that need to be set. Tip: write these to a file that you can source later (e.g. `source angora.env`).

Also install the required tools (`gclang`) using `build/install_tools.sh`.

```bash
# You might need to change this one to point to your LLVM install path
source parmesan.env
build/build.sh
export PARMESAN_BASE=$(pwd)
```

## 2) Get sources
```bash
# Create a workdir
mkdir workdir
cd workdir
wget http://ftpmirror.gnu.org/binutils/binutils-2.34.tar.xz
# or curl -O http://ftpmirror.gnu.org/binutils/binutils-2.34.tar.xz
tar xf binutils-2.34.tar.xz
mkdir build # Create a build dir
```

## 3) Build bitcode file using gclang
```bash
cd binutils-2.34
CC=gclang CXX=gclang++ CFLAGS="-g -fPIC" ./configure --with-pic
make -j$(nprocs) # Build in parallel
cd binutils/
get-bc objdump
# Will create the file objdump.bc
mkdir -p ../../build
cp objdump.bc ../../build
cd ../../build
```

## 4) Gather targets using Static Analyzers

Use the Dockerfile in `static_analysis`to build a Docker image containing several static analyzers.

```bash
cd /static_analysis
docker build -t static_analyzers .
```

Use the Rust tool in `/parser` to parse a SARIF report into a `filename:line` formatted `custom_targets.txt` file ready to be pruned by the script in `/misc/prune_targets.py` or to be fed to the fuzzer.

## 5) Run StaticFuzz pipeline

ParmeSan includes a script `tools/build_bc.py` that runs the many commands required to get the targets and build the different target binaries.

StaticFuzz includes a similar script `tools/build_bc.py` that looks for a file in the format `.custom_targets.txt` (eg `objdump.custom_targets.txt`) which contains the targets found by the static analyzers in a `filename:line` format.
This is used to statically gather the conditionals which the fuzzer has to target to reach potential buggy locations in code.

For `objdump`, you can, for example, use the `-s -d` flags. Also add `@@` in place where the input file would normally go. So the flags for objdump become `-s -d @@`. If no arguments are given, it will default to just `@@`.

The script also expects a folder called `in/` with some inputs used for profiling the target application.

```bash
mkdir in/
# Get some input seeds for objdump
cp /usr/bin/whoami in/
# Add small dummy file
echo "AAAAAAAA" > in/a.txt
# Build everything
python3 $PARMESAN_BASE/tools/compile_bc_custom.py objdump.bc -s -d @@
# Will take a long time, go get a coffee or a beer
# ...
# After some time it will print the command you can use
# to start the fuzzing. 
```

## 6) Start fuzzing
Now you can start fuzzing using the command printed in the previous step.

```bash
# Something like: 
/path/to/parmesan/bin/fuzzer -c ./targets.json -i in -o out -t ./objdump.track -s ./objdump.san.fast -- ./objdump.fast -s -d @@
```

This should start up the fuzzer (with the sanopt optimization), and show you something like the following:

![ParmeSan Screenshot](/misc/screenshot.png)


If you do not want to fuzz it with a sanitizer enable at all, remove the `-s objdump.san.fast` flag. Alternatively, you can also fuzz the target with the sanitizer always enabled. Simply replace `objdump.fast` with `objdump.san.fast` in that case.

## 7) Analyze code coverage

[Evaluate coverage](./docs/coverage.md)

To check whether the fuzzer has reached targets and with what test case run use the `/tools/search_lines.py` script based on `afl-cov`.

```bash
python3 /tools/search_lines.py /path/to/out /path/to/gcovsupported-binary /path/to/codedir
```

