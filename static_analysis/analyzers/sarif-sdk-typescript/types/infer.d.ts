// https://github.com/facebook/infer/blob/master/infer/src/atd/jsonbug.atd

export type json_trace_item = {
  level : number;
  filename : string;
  line_number : number;
  column_number : number;
  description : string;
  node_tags: node_tag[];
}

export type node_tag = {
  tag: string;
  value: string;
}

export type loc = {
  file: string;
  lnum: number;
  cnum: number;
  enum: number;
}

export type jsonbug = {
  bug_class : string;
  kind : string;
  bug_type : string;
  doc_url? : string;
  qualifier : string;
  severity : string;
  visibility : string;
  line: number;
  column: number;
  procedure : string;
  procedure_id : string;
  procedure_start_line : number;
  file : string;
  bug_trace : json_trace_item[];
  key : string;
  node_key : string;
  hash : string;
  dotty? : string;
  infer_source_loc?: loc;
  bug_type_hum: string;
  linters_def_file?: string;
  traceview_id?: number;
  censored_reason : string;
  access? : string;
}

export type report = jsonbug[]

export type json_trace = {
  trace : json_trace_item[];
}