
import {Sarif, Run, Result, Rule, CodeFlow, ThreadFlow, ThreadFlowLocation, File, Location} from "./sarif/Sarif2"
import * as path from 'path';
import Uri from "vscode-uri"
import * as fs from 'fs'
import * as mime from 'mime'
import * as md5 from 'md5';
import * as plist from "./types/plist"
import Converter from './Converter';
var plistParser = require('plist');


export default class CSAPlistConverter extends Converter {

    _project_path: string;
    _current_input: plist.PlistJson;
    _output: Sarif;

    _files: Map<string,File> = new Map<string,File>();

    constructor(project_path: string, computeMd5: boolean) {
        super(project_path, computeMd5);
    }

    public convert(input: string) {
        this._current_input = plistParser.parse(input);
        this._files.clear();
        let run: Run = {
            tool: {
		        driver : {
                    name: this._current_input.clang_version,
		        },
            },
            results: [],
            resources: {
                rules: {},
            },
        };
        this._current_input.diagnostics.forEach(diagnostic => {
            // create the Rule object if it doesn't already exist
            if (!(diagnostic.check_name in run.resources.rules)) {
                let rule : Rule = {
                    id: diagnostic.check_name,
                    name: {
                        text: diagnostic.check_name,
                    }
                }
                run.resources.rules[diagnostic.check_name] = rule;
            }
            // create the Result object
            let res : Result = {
                message: {
                    text: diagnostic.description,
                },
                ruleId: diagnostic.check_name,
                codeFlows: [this.genCodeFlow(diagnostic.path)],
                locations: [{
                    physicalLocation: {
                        artifactLocation: {
                            uri: this.getUri(this._current_input.files[diagnostic.location.file]),
                        },
                        region: this.genRegion(diagnostic.location)                    
                    }
                }] 
            };
            run.results.push(res);
        });
        // make sure we create a File object for each file mentioned in the plist file
        this._current_input.files.forEach(filename => {
            this.getUri(filename);
        });
        // adding files that appear in results
        run.files = {};
        this._files.forEach((file,name) => {
            run.files[name] = file;
        });

        this._output.runs.push(run);
    }

    public generateOutput(outputFileName: string) {
        let stringOutput = JSON.stringify(this._output, null, 2);
        if (outputFileName) {
            fs.writeFileSync(outputFileName,stringOutput);
        } else {
            console.log(stringOutput);
        }
    }

    private genCodeFlow(trace: plist.PathStep[]): CodeFlow {
        let locations: ThreadFlowLocation[] = [];
        let currentStep = 1;
        let previousDepth = 0;
        let previousStep: ThreadFlowLocation = undefined;
        trace.forEach(pathstep => {
            if (pathstep.edges) {
                pathstep.edges.forEach(edge => {
                });
            } else {
                let step : ThreadFlowLocation = {
                    step: currentStep++,
                    location: {
                        physicalLocation: {
                            fileLocation: {
                                uri: this.getUri(this._current_input.files[pathstep.location.file]),
                            },
                            region: pathstep.ranges? this.genRegionFromRanges(pathstep.ranges) : this.genRegion(pathstep.location)
                        }
                    }
                };
                step.location.physicalLocation.region.message = {
                    text: pathstep.message
                };
                if (pathstep.depth < previousDepth) {
                    step.kind = "callReturn";
                } else if (pathstep.depth > previousDepth) {
                    previousStep.kind = "call";
                }
                locations.push(step)
                previousDepth = pathstep.depth;
                previousStep = step;
            }
        }); 
        return {
            threadFlows: [{
                locations: locations
            }]
        };
    }

    private genRegion(location: plist.Location): any {
        return {
            startLine: location.line,
            startColumn: location.col
        };
    }

    private genRegionFromRanges(plistRanges: plist.Location[][]): any {
        let region : {
            startLine?: number,
            startColumn?: number,
            endLine?: number, 
            endColumn?: number
        } = {};
        plistRanges.forEach(range => {
            let start = range[0];
            let end = range[1];
            if (!region.startLine || start.line < region.startLine) {
                region.startLine = start.line;
                region.startColumn = start.col;
            }
            if (start.line == region.startLine && start.col < region.startColumn) {
                region.startColumn = start.col;
            }
            if (!region.endLine || end.line > region.endLine) {
                region.endLine = end.line
                region.endColumn = end.col
            }
            if (end.line == region.endLine && end.col > region.endColumn) {
                region.endColumn = end.col
            }
        });
        return region;
    }

    private getUri(file: string): string {
        let absolutePath = path.join(this._project_path,file);
        if (!fs.existsSync(absolutePath)) {
            absolutePath = file;
        }
        let uri = Uri.file(absolutePath);
        let stringUri = uri.toString();
        if (!(stringUri in this._files)) {
            this._files.set(stringUri,{
                fileLocation: {
                    uri: stringUri,
                },
                mimeType: mime.getType(stringUri),
                hashes: (this._computeMd5 && fs.existsSync(uri.fsPath)) ? [{
                    value: md5(fs.readFileSync(uri.fsPath)),
                    algorithm: 'md5'
                }]: undefined
            });
        }
        return stringUri;
    }
}
