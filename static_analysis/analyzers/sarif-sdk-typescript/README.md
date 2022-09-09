# Description
This repo contains code to convert output from various static
analysis tools to SARIF. Two input formats are supported, both
of which happen to be JSON:

* Plist, supported by the Clang Static Analyzer and Cppcheck,
* The Facebook Infer format.

Nothing herein takes care of running those tools. This code will only convert their
output files to SARIF.

This version supports the version 2 of SARIF as defined in
[Committee Specification Draft 1] (https://github.com/oasis-tcs/sarif-spec/blob/735f29242a5a0d533eaa8234e6cbc3257d632344/Documents/CommitteeSpecificationDrafts/CSD.1/sarif-schema.json).
Support for subsequent versions may become available.


# Installation

Installation instructions are given for Linux systems only.

You need Node.js and npm installed. You also need the Typescript compiler (tsc). If they are not:
```
curl -sL https://deb.nodesource.com/setup_8.x | sudo -E bash -
sudo apt-get install -y nodejs
sudo npm install -g typescript
```


Then, in the directory where you have cloned the repo:
```
npm install
tsc -p tsconfig.json
```

# Running the converters

```
node out/main.js --help
```

# Modifying

If you wish to modify these converters, note that the Typescript
object models in the directory sarif are automatically derived
from the JSON schema specification using [json-schema-to-typescript]
(https://www.npmjs.com/package/json-schema-to-typescript).
To be self-contained, this repo contains a copy of the specification.

# Troubleshooting

If you get errors when installing typescript, try:
```
npm config set strict-ssl false
```

Sometimes, you can also have issues with [npm permissions]
(https://docs.npmjs.com/getting-started/fixing-npm-permissions).

