export interface PlistJson {
  clang_version?: string;
  files?: string[];
  diagnostics?: Diagnostic[];
  [k: string]: any;
}

export interface Diagnostic {
    path?: PathStep[];
    description?: string;
    category?: string;
    type?: string;
    check_name?: string;
    issue_hash_content_of_line_in_context?: string;
    issue_context_kind?: string;
    issue_context?: string;
    issue_hash_function_offset?: string;
    location?: Location;
    [k: string]: any;
  }

export interface PathStep {
      kind?: "control"| "event";
      edges?: {
        start?: Location[];
        end?: Location[];
        [k: string]: any;
      }[];
      location?: Location;
      ranges?: Location[][];
      message?: string;
      depth?: number;
      extended_message?: string;
      [k: string]: any;
    }

export interface Location {
      line?: number;
      col?: number;
      file?: number;
      [k: string]: any;
    }