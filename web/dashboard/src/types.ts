export interface DnsResolutionEvent {
  fqdn: string;
  dns_server_ip: string;
  dns_server_name?: string | null;
  dns_server_country: string;
  resolved_ip?: string | null;
  success: boolean;
  duration_ms: number;
  finished_at: string;
  error?: string | null;
}

export interface Filters {
  domain: string;
  country: string;
  status: "" | "success" | "failure";
  from: string;
  to: string;
}
