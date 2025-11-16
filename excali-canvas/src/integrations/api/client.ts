/* GraphFlow API client */
// Try to read backend base URL from the OpenAPI spec if available
// Note: The spec should define a `servers` array. If it's missing, we fall back to env or window origin.
// Import path goes up to the repo root where openapi.json lives.
// If this import ever fails, Vite will tree-shake it, and we'll continue with env/window fallback.
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - allow JSON import without a type
import openapi from "../../../../openapi.json";

type OpenAPISpec = {
  servers?: Array<{ url: string }>
};

export interface GenerateRequest {
  content: string;
  // Optional: tier can be kept if your backend wants it; otherwise omit on server side
  tier?: "pro" | "free";
  allow_images?: boolean;
}

export interface GenerateResponse {
  // Based on openapi.json: expects graph_data and optional scene
  graph_data: unknown;
  scene?: unknown;
}

export interface RenderRequest {
  // Render can accept a scene or graph data according to openapi.json
  scene?: unknown;
  graph_data?: unknown;
  format?: "png" | "svg";
}

export interface RenderResponse {
  // Expect URLs or base64 data depending on your backend contract
  png_url?: string;
  svg_url?: string;
  png_base64?: string;
  svg?: string;
}

const spec = (openapi as OpenAPISpec) || {};
const SPEC_BASE_URL = Array.isArray(spec.servers) && spec.servers.length > 0 ? spec.servers[0].url : undefined;
const API_BASE_URL =
  (SPEC_BASE_URL && SPEC_BASE_URL.trim()) ||
  import.meta.env.VITE_GRAPHFLOW_API_URL ||
  window.location.origin;

async function jsonFetch<T>(url: string, init: RequestInit): Promise<T> {
  const res = await fetch(url, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init.headers || {}),
    },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`Request failed ${res.status}: ${text || res.statusText}`);
  }
  return (await res.json()) as T;
}

export async function generateGraph(payload: GenerateRequest): Promise<GenerateResponse> {
  const url = `${API_BASE_URL}/graph/generate`;
  return jsonFetch<GenerateResponse>(url, {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export async function renderScene(payload: RenderRequest): Promise<RenderResponse> {
  const url = `${API_BASE_URL}/graph/render`;
  return jsonFetch<RenderResponse>(url, {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export const api = {
  generateGraph,
  renderScene,
};
