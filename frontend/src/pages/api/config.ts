import type { APIRoute } from 'astro';

export const prerender = false;

export const GET: APIRoute = async () => {
  // Read environment variables at runtime
  const config = {
    AUTH_SERVICE: process.env.AUTH_SERVICE || process.env.PUBLIC_AUTH_SERVICE || 'http://localhost:8001',
    PRODUCT_SERVICE: process.env.PRODUCT_SERVICE || process.env.PUBLIC_PRODUCT_SERVICE || 'http://localhost:8002',
    ORDER_SERVICE: process.env.ORDER_SERVICE || process.env.PUBLIC_ORDER_SERVICE || 'http://localhost:8003',
  };

  return new Response(JSON.stringify(config), {
    status: 200,
    headers: {
      'Content-Type': 'application/json',
      'Cache-Control': 'no-cache, no-store, must-revalidate',
    },
  });
};