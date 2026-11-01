// Shared document and editing types. The framework-independent core owns their rules.
export type Point = { x: number; y: number };
export type FieldValue = string | number | boolean;
export type GraphNode = {
  id: string;
  kind: string;
  title: string;
  position: Point;
  values: Record<string, FieldValue>;
  note: string;
  frame: string | null;
};
export type Endpoint = { node: string; port: string };
export type GraphEdge = { id: string; source: Endpoint; target: Endpoint };
export type Frame = { id: string; title: string };
export type GraphDocument = {
  format: string;
  version: number;
  id: string;
  name: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  frames: Frame[];
};
export type Fragment = {
  format: string;
  version: number;
  nodes: GraphNode[];
  edges: GraphEdge[];
  frames: Frame[];
};
export type Port = {
  id: string;
  label: string;
  valueType: string;
  multiple: boolean;
  required: boolean;
};
export type Field = {
  id: string;
  label: string;
  valueType: string;
  default: FieldValue;
  options?: { value: string; label: string }[];
  integer?: boolean;
  unit?: "seconds";
  min?: number;
  max?: number;
  multiline?: boolean;
};
export type Definition = {
  kind: string;
  label: string;
  category: string;
  role?: string;
  description: string;
  inputs: Port[];
  outputs: Port[];
  fields: Field[];
  width: number;
  height: number;
};
export type Diagnostic = {
  code: string;
  nodeId: string;
  portId: string | null;
  message: string;
};
export type FrameRect = {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
};
export type Snapshot = {
  doc: GraphDocument;
  epoch: number;
  revision: number;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
  undoCount: number;
  diagnostics: Diagnostic[];
  frameRects: FrameRect[];
};
export type Exported = { token: string; json: string; filename: string };
export type Version = { epoch: number; revision: number };
export type Anchor = { endpoint: Endpoint; output: boolean };
export type Command =
  | { type: "add"; kind: string; position: Point; link?: Anchor }
  | { type: "move"; positions: { id: string; position: Point }[] }
  | {
      type: "setNode";
      id: string;
      title?: string;
      note?: string;
      values?: Record<string, FieldValue>;
    }
  | { type: "rename"; name: string }
  | { type: "delete" | "disconnect" | "duplicate" | "ungroup"; ids: string[] }
  | {
      type: "connect";
      source: Endpoint;
      target: Endpoint;
      replaceEdge?: string;
    }
  | { type: "paste"; fragment: Fragment; position: Point }
  | { type: "group"; ids: string[]; title: string }
  | { type: "renameFrame"; id: string; title: string }
  | { type: "reroute"; edgeId: string; position?: Point };
