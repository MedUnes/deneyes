import { useEffect, useMemo, useState } from "react";
import dayjs from "dayjs";
import { Filters, DnsResolutionEvent } from "./types";
import { FiltersForm } from "./components/Filters";
import { ResolutionTable } from "./components/ResolutionTable";
import { ResolutionChart } from "./components/ResolutionChart";

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8080";

const initialFilters: Filters = {
  domain: "",
  country: "",
  status: "",
  from: dayjs().subtract(1, "hour").format("YYYY-MM-DDTHH:mm"),
  to: ""
};

export default function App() {
  const [filters, setFilters] = useState<Filters>(initialFilters);
  const [data, setData] = useState<DnsResolutionEvent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      if (filters.domain) params.set("domain", filters.domain);
      if (filters.country) params.set("country", filters.country);
      if (filters.status) params.set("status", filters.status);
      if (filters.from) params.set("from", new Date(filters.from).toISOString());
      if (filters.to) params.set("to", new Date(filters.to).toISOString());
      params.set("limit", "200");

      const response = await fetch(`${API_BASE}/api/v1/dns/resolutions?${params.toString()}`);
      if (!response.ok) {
        throw new Error(`API request failed: ${response.status}`);
      }
      const payload: DnsResolutionEvent[] = await response.json();
      setData(payload);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filters.domain, filters.country, filters.status, filters.from, filters.to]);

  const countries = useMemo(() => {
    const set = new Set<string>();
    data.forEach((event) => set.add(event.dns_server_country));
    return Array.from(set).sort();
  }, [data]);

  return (
    <main>
      <header>
        <div>
          <h1>DNeyeS Radar</h1>
          <p>Visualise DNS resolution history stored in ClickHouse.</p>
        </div>
      </header>
      <FiltersForm
        filters={filters}
        onChange={setFilters}
        onRefresh={fetchData}
        loading={loading}
        availableCountries={countries}
      />
      {error && (
        <div className="card" style={{ color: "#b91c1c" }}>
          <strong>Failed to fetch data:</strong> {error}
        </div>
      )}
      <ResolutionChart data={data} />
      <ResolutionTable data={data} />
    </main>
  );
}
