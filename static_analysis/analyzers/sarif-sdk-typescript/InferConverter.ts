import {Sarif, Run, Result, Rule, CodeFlow, ThreadFlow, ThreadFlowLocation, File, Location} from "./sarif/Sarif2"
import * as path from 'path';
import {report, json_trace_item} from "./types/infer";
import Uri from "vscode-uri";
import * as fs from 'fs';
import * as mime from 'mime';
import * as md5 from 'md5';
import Converter from './Converter';

export default class InferConverter extends Converter {

    _current_input: report;
    _output: Sarif;

    _files: Map<string,File> = new Map<string,File>();

    constructor(project_path: string, computeMd5: boolean) {
        super(project_path, computeMd5);
    }

    public convert(input: string): void {
        this._current_input = JSON.parse(input);
        this._files.clear();
        let run: Run = {
            tool: {
	            driver: {
		            name: "Infer"
		        },
            },
            results: [],
            resources: {
                rules: {}
            }
        };
        this._output.runs.push(run);
        this._current_input.forEach(k => {
            // create the Rule object if it doesn't already exist
            if (!(k.bug_type in run.resources.rules)) {
                let rule : Rule = {
                    id: k.bug_type,
                    name: {
                        text: k.bug_type_hum
                    }
                }
                run.resources.rules[k.bug_type] = rule;
            }
            // create the Result object
            let res : Result = {
                message: {
                    text: k.qualifier,
                },
                level: this.kindToLevel(k.kind),
                ruleId: k.bug_type,
                codeFlows: [this.bugTraceToCodeFlow(k.bug_trace)],
                // there is a single location in an Infer report
                locations: [{ 
                    physicalLocation: {
                        artifactLocation: {
                            uri: this.getUri(k.file)
                        },
                        region: {
                            startLine: k.line,
                            startColumn: k.column
                        }
                    }
                }]
            };
            run.results.push(res);
        });
        // adding files that appear in results
        run.files = {};
        this._files.forEach((file,name) => {
            run.files[name] = file;
        });
    }

    public generateOutput(outputFileName: string) {
        let stringOutput = JSON.stringify(this._output, null, 2);
        if (outputFileName) {
            fs.writeFileSync(outputFileName,stringOutput);
        } else {
            console.log(stringOutput);
        }
    }

    private kindToLevel(kind: string): "warning" | "error" {
        switch(kind) {
            case "ERROR":
                return "error";
            default:
                return "warning";
        }
    }

    private bugTraceToCodeFlow(trace: json_trace_item[]): CodeFlow {
        let threadFlow: ThreadFlow = {
            locations: [],
        }
        let curStep = 1;
        let previousItem: json_trace_item = undefined;
        let previousLoc: ThreadFlowLocation = undefined;
        let nextIsCallReturn = false;
        trace.forEach(item => {
            let loc: ThreadFlowLocation = {
                step: curStep++,
                location: {
                    physicalLocation: {
                        fileLocation: {
                            uri: this.getUri(item.filename),
                        },
                        region: {
                            startLine: item.line_number,
                            startColumn: item.column_number,
                            message: {
                                text: item.description,
                            }
                        },
                    },
                }
            }
            if (nextIsCallReturn) {
                loc.kind = 'callReturn';
            }
            nextIsCallReturn = false;
            if (item.node_tags) {
            item.node_tags.forEach(nodeTag => {
                if (nodeTag.tag == 'kind') {
                    switch (nodeTag.value) {
                        case 'procedure_end':
                            loc.kind = 'functionExit';
                            nextIsCallReturn = true;
                            break;
                        case 'procedure_start':
                            loc.kind = 'functionEnter';
                            if (previousLoc) {
                                previousLoc.kind = 'call';
                            }
                            break;
                        case 'branch':
                            loc.kind = 'branch';
                            break;
                    }
                }
            });
            }
            threadFlow.locations.push(loc);
            previousItem = item;
            previousLoc = loc;
        });
        let codeFlow: CodeFlow = {
            threadFlows: [threadFlow] 
        };
        return codeFlow
    }

    private getUri(file: string): string {
        let absolutePath = path.join(this._project_path,file);
        let uri = Uri.file(absolutePath);
        let stringUri = uri.toString();
        if (!(stringUri in this._files)) {
            let file: File = {
                fileLocation: {
                    uri: stringUri,
                },
                mimeType: mime.getType(stringUri),
            };
            if (this._computeMd5 && fs.existsSync(uri.fsPath)) {
                file.hashes = [{
                    value: md5(fs.readFileSync(uri.fsPath)),
                    algorithm: 'md5'
                }];
            }
            this._files.set(stringUri,file);
        }
        return stringUri;
    }
}
