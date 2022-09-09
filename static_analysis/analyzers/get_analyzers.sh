#!/bin/bash

#Pull LLVM13 Ubuntu20.04 x86-64 from GitHub Repo 
wget -q https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04.tar.xz
tar -xf clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04.tar.xz
rm clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04.tar.xz
mv clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04 /usr/llvm

echo "export LD_LIBRARY_PATH=/usr/llvm/lib:\$LD_LIBRARY_PATH" >> ~/.bashrc
echo "export PATH=/usr/llvm/bin:\$PATH" >> ~/.bashrc

#Pull Infer 1.1.0
curl -sSL "https://github.com/facebook/infer/releases/download/v1.1.0/infer-linux64-v1.1.0.tar.xz" | tar -C /opt -xJ 
ln -s "/opt/infer-linux64-v1.1.0/bin/infer" /usr/local/bin/infer

#Pull Infer converter
npm install -g typescript
#git clone https://github.com/sarif-standard/sarif-sdk-typescript.git
cd sarif-sdk-typescript
npm install
tsc -p tsconfig.json
cd ..

#Pull CodeQL
mkdir codeql-home
cd codeql-home
wget -q https://github.com/github/codeql-cli-binaries/releases/download/v2.5.4/codeql-linux64.zip
git clone https://github.com/github/codeql.git codeql-repo
unzip codeql-linux64.zip
cd ..

#Pull Cppcheck
wget -q https://github.com/danmar/cppcheck/archive/2.4.1.zip
unzip 2.4.1.zip
cd cppcheck-2.4.1
make
cd ..

#Pull CPAChecker
wget -q https://cpachecker.sosy-lab.org/CPAchecker-2.0-unix.zip
unzip CPAchecker-2.0-unix.zip
