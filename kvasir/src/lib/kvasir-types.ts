export interface FileTreeEntry {
  name: string;
  path: string;
  is_dir: boolean;
  extension: string | null;
  size_bytes: number;
}

export interface FileContent {
  path: string;
  content: string;
  language: string;
  line_count: number;
  size_bytes: number;
}

export interface FormatConversion {
  content: string;
  token_count: number;
}

export interface AllFormats {
  json: FormatConversion;
  yaml: FormatConversion;
  toml: FormatConversion;
  toon: FormatConversion;
  ron: FormatConversion;
  source_format: string;
}

export interface JsonlInfo {
  path: string;
  entry_count: number;
  size_bytes: number;
}

export interface JsonlEntry {
  index: number;
  content: string;
  entry_count: number;
}

export interface TableData {
  path: string;
  headers: string[];
  rows: string[][];
  row_count: number;
  column_count: number;
  source_format: string;
  size_bytes: number;
}

export interface KvasTreeNode extends FileTreeEntry {
  expanded: boolean;
  children: KvasTreeNode[];
  loading: boolean;
}

export type ViewTab = "code" | "data" | "preview" | "inspect" | "jsonl" | "table";
export type DataFormat = "json" | "yaml" | "toml" | "toon" | "ron";
export type WrapMode = "nowrap" | "wrap79" | "wrapwidth";
