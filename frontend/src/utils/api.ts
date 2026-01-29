// Runtime configuration
interface RuntimeConfig {
  AUTH_SERVICE: string;
  PRODUCT_SERVICE: string;
  ORDER_SERVICE: string;
}

let configCache: RuntimeConfig | null = null;
let configPromise: Promise<RuntimeConfig> | null = null;

// Check if we're running on the server (Node.js)
function isServer(): boolean {
  return typeof process !== 'undefined' && process.versions != null && process.versions.node != null;
}

// Get config from process.env (server-side only)
function getServerConfig(): RuntimeConfig {
  // Safely access process.env only on server
  const env = typeof process !== 'undefined' ? process.env : {};
  return {
    AUTH_SERVICE: env.AUTH_SERVICE || env.PUBLIC_AUTH_SERVICE || "http://localhost:8001",
    PRODUCT_SERVICE: env.PRODUCT_SERVICE || env.PUBLIC_PRODUCT_SERVICE || "http://localhost:8002",
    ORDER_SERVICE: env.ORDER_SERVICE || env.PUBLIC_ORDER_SERVICE || "http://localhost:8003",
  };
}

// Fetch runtime configuration from API endpoint or process.env
async function getConfig(): Promise<RuntimeConfig> {
  // If running on server, read directly from process.env
  if (isServer()) {
    return getServerConfig();
  }

  // Client-side: return cached config if available
  if (configCache) {
    return configCache;
  }

  // Return existing promise if already fetching
  if (configPromise) {
    return configPromise;
  }

  // Fetch config from API endpoint
  configPromise = fetch('/api/config')
    .then(res => res.json())
    .then(config => {
      configCache = config;
      configPromise = null;
      return config;
    })
    .catch(error => {
      console.error('Failed to fetch runtime config:', error);
      configPromise = null;
      // Return defaults if fetch fails
      return {
        AUTH_SERVICE: "http://localhost:8001",
        PRODUCT_SERVICE: "http://localhost:8002",
        ORDER_SERVICE: "http://localhost:8003",
      };
    });

  return configPromise;
}

export interface Order {
  id: number;
  product_id: number;
  buyer_id: number;
  seller_id: number;
  status: string;
  buyer_location: LocationInfo;
  seller_location: LocationInfo;
  midpoint_info: MidpointInfo;
}

export interface LocationInfo {
  latitude: number;
  longitude: number;
  address: string;
}

export interface MidpointInfo {
  midpoint: {
    latitude: number;
    longitude: number;
  };
  distance_to_buyer_km: number;
  distance_to_seller_km: number;
  total_distance_km: number;
}

export interface GeocodeResult {
  latitude: number;
  longitude: number;
  address: string;
}

export interface LocationUpsertResponse {
  id: number;
  user_id: number;
  latitude: number;
  longitude: number;
  address: string;
}

export interface Product {
  id: number;
  seller_id: number;
  category_id: number;
  category_name: string;
  title: string;
  description: string;
  price: number;
  image_url?: string;
  status: string;
}

export interface Category {
  id: number;
  name: string;
  slug: string;
  icon?: string;
}

export interface User {
  id: number;
  email: string;
  name: string;
  email_verified: boolean;
}

export interface AuthResponse {
  token: string;
  user: User;
}

// Auth API
export async function register(email: string, password: string, name: string) {
  const config = await getConfig();
  const response = await fetch(`${config.AUTH_SERVICE}/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password, name }),
  });
  if (!response.ok) throw new Error("Registration failed");
  return response.json();
}

export async function verifyEmail(email: string, code: string) {
  const config = await getConfig();
  const response = await fetch(`${config.AUTH_SERVICE}/verify-email`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, code }),
  });
  if (!response.ok) throw new Error("Verification failed");
  return response.json();
}

export async function login(
  email: string,
  password: string,
): Promise<AuthResponse> {
  const config = await getConfig();
  const response = await fetch(`${config.AUTH_SERVICE}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  if (!response.ok) throw new Error("Login failed");
  return response.json();
}

export async function getMe(token: string): Promise<User> {
  const config = await getConfig();
  const response = await fetch(`${config.AUTH_SERVICE}/me`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) throw new Error("Failed to get user");
  return response.json();
}

// Product API
export async function getCategories(): Promise<Category[]> {
  const config = await getConfig();
  const response = await fetch(`${config.PRODUCT_SERVICE}/categories`);
  if (!response.ok) throw new Error("Failed to fetch categories");
  return response.json();
}

export async function getProducts(
  categoryId?: number,
  limit = 20,
): Promise<Product[]> {
  const config = await getConfig();
  const params = new URLSearchParams();
  if (categoryId) params.append("category_id", categoryId.toString());
  params.append("limit", limit.toString());

  const response = await fetch(`${config.PRODUCT_SERVICE}/products?${params}`);
  if (!response.ok) throw new Error("Failed to fetch products");
  return response.json();
}

export async function getProduct(id: number): Promise<Product> {
  const config = await getConfig();
  const response = await fetch(`${config.PRODUCT_SERVICE}/products/${id}`);
  if (!response.ok) throw new Error("Failed to fetch product");
  return response.json();
}

export async function getCategoryProducts(
  slug: string,
  limit = 20,
): Promise<Product[]> {
  const config = await getConfig();
  const response = await fetch(
    `${config.PRODUCT_SERVICE}/categories/${slug}/products?limit=${limit}`,
  );
  if (!response.ok) throw new Error("Failed to fetch category products");
  return response.json();
}

export async function createProduct(
  token: string,
  data: {
    category_id: number;
    title: string;
    description: string;
    price: number;
    image_url?: string;
  },
): Promise<Product> {
  const config = await getConfig();
  const response = await fetch(`${config.PRODUCT_SERVICE}/products`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create product");
  return response.json();
}

// Order API
export async function createOrder(
  token: string,
  data: {
    product_id: number;
    seller_id: number;
    buyer_location: {
      latitude: number;
      longitude: number;
      address: string;
    };
  },
) {
  const config = await getConfig();
  const response = await fetch(`${config.ORDER_SERVICE}/orders`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create order");
  return response.json();
}

export async function getOrder(token: string, id: number): Promise<Order> {
  const config = await getConfig();
  const response = await fetch(`${config.ORDER_SERVICE}/orders/${id}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) throw new Error("Failed to fetch order");
  return response.json();
}

export async function getMyOrders(token: string): Promise<Order[]> {
  const config = await getConfig();
  const response = await fetch(`${config.ORDER_SERVICE}/orders/my-orders`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) throw new Error("Failed to fetch orders");
  return response.json();
}

// Geocode API
export async function geocodeAddress(address: string): Promise<GeocodeResult> {
  const config = await getConfig();
  const response = await fetch(`${config.ORDER_SERVICE}/geocode/address`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ address }),
  });
  if (!response.ok) throw new Error("Geocoding failed");
  return response.json();
}

export async function reverseGeocode(
  latitude: number,
  longitude: number,
): Promise<GeocodeResult> {
  const config = await getConfig();
  const response = await fetch(`${config.ORDER_SERVICE}/geocode/reverse`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ latitude, longitude }),
  });
  if (!response.ok) throw new Error("Reverse geocoding failed");
  return response.json();
}

export async function upsertMyLocation(
  token: string,
  data: { latitude: number; longitude: number; address: string },
): Promise<LocationUpsertResponse> {
  const config = await getConfig();
  const response = await fetch(`${config.ORDER_SERVICE}/locations/me`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to save location");
  return response.json();
}

// Local storage helpers
export function saveToken(token: string) {
  localStorage.setItem("auth_token", token);
}

export function getToken(): string | null {
  return localStorage.getItem("auth_token");
}

export function clearToken() {
  localStorage.removeItem("auth_token");
}

export function saveUser(user: User) {
  localStorage.setItem("user", JSON.stringify(user));
}

export function getUser(): User | null {
  const user = localStorage.getItem("user");
  return user ? JSON.parse(user) : null;
}

export function clearUser() {
  localStorage.removeItem("user");
}
