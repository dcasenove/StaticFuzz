import {Sarif} from "./sarif/Sarif2"


export default abstract class Converter {
    
    _project_path: string;
    _output: Sarif;
    _computeMd5: boolean

    protected constructor(project_path: string, computeMd5: boolean) {
        this._project_path = project_path;
        this._computeMd5 = computeMd5;
        this._output = { 
            $schema: "http://json-schema.org/draft-04/schema#",
            version: "2.0.0",
            runs: []
        };
    }
    
    public abstract convert(data: string): void;

    public abstract generateOutput (outputFileName: string): void;
}