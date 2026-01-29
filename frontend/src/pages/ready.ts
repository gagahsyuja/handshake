import type { APIRoute } from "astro";

export const prerender = false;

type CheckResult = {
  name: string;
  url: string;
  ok: boolean;
  status?: number;
  latency_ms?: number;
  error?: string;
};

function envUrl(key: string): string | undefined {
  const raw = process.env[key];
  if (typeof raw !== "string") return undefined;
  const trimmed = raw.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function joinUrl(base: string, path: string): string {
  // Ensure exactly one slash between base and path.
  const b = base.endsWith("/") ? base.slice(0, -1) : base;
  const p = path.startsWith("/") ? path : `/${path}`;
  return `${b}${p}`;
}

async function fetchWithTimeout(url: string, timeoutMs: number): Promise<Response> {
  const controller = new AbortController();
  const t = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(url, {
      method: "GET",
      headers: { accept: "application/json" },
      signal: controller.signal,
    });
  } finally {
    clearTimeout(t);
  }
}

async function checkService(
  name: string,
  baseUrl: string | undefined,
  timeoutMs: number
): Promise<CheckResult> {
  if (!baseUrl) {
    return {
      name,
      url: "",
      ok: false,
      error: `Missing env var for ${name} base URL`,
    };
  }

  const url = joinUrl(baseUrl, "/ready");
  const start = Date.now();

  try {
    const res = await fetchWithTimeout(url, timeoutMs);
    const latencyMs = Date.now() - start;

    // Consider any 2xx as ready; most services should return 200 or 503.
    const ok = res.ok;

    return {
      name,
      url,
      ok,
      status: res.status,
      latency_ms: latencyMs,
    };
  } catch (e: unknown) {
    const latencyMs = Date.now() - start;
    const message =
      e instanceof Error ? e.message : typeof e === "string" ? e : "Unknown error";

    return {
      name,
      url,
      ok: false,
      latency_ms: latencyMs,
      error: message,
    };
  }
}

export const GET: APIRoute = async () => {
  // These are provided by `.env` or system environment:
  // AUTH_SERVICE=http://localhost:8001
  // PRODUCT_SERVICE=http://localhost:8002
  // ORDER_SERVICE=http://localhost:8003
  //
  // Note: EMAIL_SERVICE is not checked here as it's already checked by auth-service
  //
  // Also supports legacy PUBLIC_* prefix for backward compatibility
  const authBase = envUrl("AUTH_SERVICE") || envUrl("PUBLIC_AUTH_SERVICE");
  const productBase = envUrl("PRODUCT_SERVICE") || envUrl("PUBLIC_PRODUCT_SERVICE");
  const orderBase = envUrl("ORDER_SERVICE") || envUrl("PUBLIC_ORDER_SERVICE");

  const timeoutMs = (() => {
    const v = envUrl("READY_TIMEOUT_MS");
    const n = v ? Number(v) : NaN;
    return Number.isFinite(n) && n > 0 ? n : 1500;
  })();

  const checks = await Promise.all([
    checkService("auth-service", authBase, timeoutMs),
    checkService("product-service", productBase, timeoutMs),
    checkService("order-service", orderBase, timeoutMs),
  ]);

  const allOk = checks.every((c) => c.ok);

  return new Response(
    JSON.stringify({
      status: allOk ? "ok" : "down",
      service: "frontend",
      timeout_ms: timeoutMs,
      checks,
    }),
    {
      status: allOk ? 200 : 503,
      headers: {
        "content-type": "application/json; charset=utf-8",
        "cache-control": "no-store",
      },
    }
  );
};
