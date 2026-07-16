export interface TransformOptions {
  filename?: string;
  moduleName?: string;
  generate?: "dom" | "ssr" | "universal" | "dynamic";
  hydratable?: boolean;
  dev?: boolean;
  sourceMap?: boolean;
  contextToCustomElements?: boolean;
  delegateEvents?: boolean;
  delegatedEvents?: string[];
  omitQuotes?: boolean;
  omitAttributeSpacing?: boolean;
  inlineStyles?: boolean;
  effectWrapper?: "effect" | false;
  wrapConditionals?: boolean;
  memoWrapper?: "memo" | false;
  staticMarker?: string;
  validate?: boolean;
  omitNestedClosingTags?: boolean;
  omitLastClosingTag?: boolean;
  builtIns?: string[];
  requireImportSource?: false | string;
  renderers?: RendererOption[];
  compilerFacts?: boolean;
}

export interface RendererOption {
  name: string;
  moduleName?: string;
  elements: string[];
}

export interface TransformResult {
  code: string;
  map?: string | null;
  executionMap?: ExecutionMap;
}

export interface SourceSpan {
  start: number;
  end: number;
}

export interface ExecutionRegion {
  span: SourceSpan;
  reason: "jsx-child" | "jsx-attribute";
}

export interface CallbackRole {
  span: SourceSpan;
  role: "event-handler";
}

export interface OwnershipRegion {
  span: SourceSpan;
  kind: string;
}

export interface JsxOperation {
  span: SourceSpan;
  kind: "insert" | "dynamic-attribute" | "event-listener";
}

export interface ExecutionMap {
  compilerFactsProtocol: 1;
  sourceHash: string;
  trackedRegions: ExecutionRegion[];
  untrackedRegions: ExecutionRegion[];
  ownershipRegions: OwnershipRegion[];
  callbackRoles: CallbackRole[];
  jsxOperations: JsxOperation[];
}

export function transform(code: string, options?: TransformOptions | null): TransformResult;
export function transformAsync(
  code: string,
  options?: TransformOptions | null
): Promise<TransformResult>;
