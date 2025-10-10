import { Filters } from "../types";

type FiltersProps = {
  filters: Filters;
  onChange: (filters: Filters) => void;
  onRefresh: () => void;
  loading: boolean;
  availableCountries: string[];
};

export function FiltersForm({
  filters,
  onChange,
  onRefresh,
  loading,
  availableCountries
}: FiltersProps) {
  const update = (patch: Partial<Filters>) => onChange({ ...filters, ...patch });

  return (
    <div className="card">
      <header>
        <h2>Filters</h2>
        <button
          type="button"
          onClick={onRefresh}
          disabled={loading}
          style={{
            padding: "0.5rem 1rem",
            borderRadius: "8px",
            border: "none",
            background: "#111827",
            color: "#fff",
            cursor: loading ? "not-allowed" : "pointer"
          }}
        >
          {loading ? "Loading..." : "Refresh"}
        </button>
      </header>
      <div className="filters">
        <label>
          Domain
          <input
            value={filters.domain}
            onChange={(event) => update({ domain: event.target.value })}
            placeholder="example.com"
          />
        </label>
        <label>
          Country
          <select
            value={filters.country}
            onChange={(event) => update({ country: event.target.value })}
          >
            <option value="">All</option>
            {availableCountries.map((code) => (
              <option key={code} value={code}>
                {code.toUpperCase()}
              </option>
            ))}
          </select>
        </label>
        <label>
          Status
          <select
            value={filters.status}
            onChange={(event) => update({ status: event.target.value as Filters["status"] })}
          >
            <option value="">All</option>
            <option value="success">Success</option>
            <option value="failure">Failure</option>
          </select>
        </label>
        <label>
          From
          <input
            type="datetime-local"
            value={filters.from}
            onChange={(event) => update({ from: event.target.value })}
          />
        </label>
        <label>
          To
          <input
            type="datetime-local"
            value={filters.to}
            onChange={(event) => update({ to: event.target.value })}
          />
        </label>
      </div>
    </div>
  );
}
