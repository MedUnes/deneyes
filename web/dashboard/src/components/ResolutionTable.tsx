import dayjs from "dayjs";
import { DnsResolutionEvent } from "../types";

type ResolutionTableProps = {
  data: DnsResolutionEvent[];
};

export function ResolutionTable({ data }: ResolutionTableProps) {
  return (
    <div className="card table-container">
      <h2>Recent resolutions</h2>
      <table>
        <thead>
          <tr>
            <th>Finished</th>
            <th>Domain</th>
            <th>Resolver</th>
            <th>Country</th>
            <th>Resolved IP</th>
            <th>Latency (ms)</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {data.map((event) => (
            <tr key={`${event.fqdn}-${event.finished_at}-${event.dns_server_ip}`}>
              <td>{dayjs(event.finished_at).format("YYYY-MM-DD HH:mm:ss")}</td>
              <td>{event.fqdn}</td>
              <td>{event.dns_server_name ?? event.dns_server_ip}</td>
              <td>{event.dns_server_country.toUpperCase()}</td>
              <td>{event.resolved_ip ?? "n/a"}</td>
              <td>{event.duration_ms}</td>
              <td>
                <span className={`badge ${event.success ? "success" : "failure"}`}>
                  {event.success ? "Success" : "Failure"}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
