import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { api } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";

const MEALS: { key: string; label: string }[] = [
  { key: "desayuno", label: t.meals.desayuno },
  { key: "colacion1", label: t.meals.colacion1 },
  { key: "comida", label: t.meals.comida },
  { key: "colacion2", label: t.meals.colacion2 },
  { key: "cena", label: t.meals.cena },
];

function todayISO(): string {
  return new Date().toISOString().slice(0, 10);
}

function addDaysISO(iso: string, days: number): string {
  const d = new Date(iso + "T00:00:00");
  d.setDate(d.getDate() + days);
  return d.toISOString().slice(0, 10);
}

export default function Historial() {
  const activeUserId = useActiveUser((s) => s.activeUserId);
  const [weekStart, setWeekStart] = useState(todayISO());

  const { data: groups } = useQuery({
    queryKey: ["groups"],
    queryFn: api.foodGroupsList,
  });
  const { data: diet } = useQuery({
    queryKey: ["diet", activeUserId, weekStart],
    queryFn: () => api.dietGet(activeUserId!, weekStart),
    enabled: activeUserId != null,
  });
  const { data: measurements } = useQuery({
    queryKey: ["measurements", activeUserId],
    queryFn: () => api.measurementsList(activeUserId!),
    enabled: activeUserId != null,
  });

  // Mediciones dentro de la semana seleccionada (del día de inicio + 6 días).
  const weekEnd = useMemo(() => addDaysISO(weekStart, 6), [weekStart]);
  const inWeek = useMemo(() => {
    if (!measurements) return [];
    return measurements.filter((m) => m.date >= weekStart && m.date <= weekEnd);
  }, [measurements, weekStart, weekEnd]);

  // Valor de porciones para (meal, groupId). Si no existe, 0.
  const portionFor = (meal: string, groupId: number): number => {
    if (!diet) return 0;
    return (
      diet.portions.find((p) => p.meal_type === meal && p.group_id === groupId)
        ?.portions ?? 0
    );
  };

  const totalPortions = useMemo(() => {
    if (!diet) return 0;
    return diet.portions.reduce((s, p) => s + (p.portions || 0), 0);
  }, [diet]);

  if (!activeUserId) {
    return (
      <div className="card border-l-4 border-l-amber-400 bg-amber-50">
        <p className="text-sm text-amber-800">
          {t.common.seleccionarUsuario} para consultar su historial.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <p className="text-sm text-mint-700">
        Elige una semana para ver las porciones registradas y las mediciones de esa semana.
      </p>

      <div className="card flex flex-wrap items-end gap-3">
        <label className="space-y-1">
          <span className="label">Semana a consultar</span>
          <input
            type="date"
            className="input"
            value={weekStart}
            onChange={(e) => setWeekStart(e.target.value || todayISO())}
          />
        </label>
        <div className="text-xs text-mint-700">
          Rango: <strong>{weekStart}</strong> → <strong>{weekEnd}</strong>
        </div>
      </div>

      {/* Porciones registradas */}
      <div className="card overflow-x-auto">
        <div className="flex items-baseline justify-between mb-3 flex-wrap gap-2">
          <h3 className="font-semibold text-mint-700">
            Porciones registradas en esa semana
          </h3>
          <span className="text-xs text-mint-700">
            Total: <strong>{totalPortions}</strong> porciones
          </span>
        </div>
        {totalPortions === 0 ? (
          <p className="text-sm text-mint-700">
            No hay porciones registradas para esa semana.
          </p>
        ) : (
          <table className="table min-w-[720px]">
            <thead>
              <tr>
                <th>{t.common.grupo}</th>
                {MEALS.map((m) => (
                  <th key={m.key}>{m.label}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {(groups ?? []).map((g) => (
                <tr key={g.id}>
                  <td className="font-medium">{g.name}</td>
                  {MEALS.map((m) => {
                    const v = portionFor(m.key, g.id);
                    return (
                      <td key={m.key} className={v > 0 ? "font-medium" : "text-mint-500"}>
                        {v}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Mediciones de esa semana */}
      <div className="card">
        <h3 className="font-semibold text-mint-700 mb-3">
          Mediciones de esa semana
        </h3>
        {inWeek.length === 0 ? (
          <p className="text-sm text-mint-700">
            No hay mediciones registradas en ese rango de fechas.
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="table min-w-[640px]">
              <thead>
                <tr>
                  <th>{t.common.fecha}</th>
                  <th>{t.measurements.peso}</th>
                  <th>{t.measurements.espalda}</th>
                  <th>{t.measurements.cintura}</th>
                  <th>{t.measurements.abdomen}</th>
                  <th>{t.measurements.cadera}</th>
                </tr>
              </thead>
              <tbody>
                {inWeek.map((m) => (
                  <tr key={m.id}>
                    <td>{m.date}</td>
                    <td>{m.weight ?? "—"}</td>
                    <td>{m.back_cm ?? "—"}</td>
                    <td>{m.waist_cm ?? "—"}</td>
                    <td>{m.abdomen_cm ?? "—"}</td>
                    <td>{m.hip_cm ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Historial completo de mediciones */}
      <div className="card">
        <h3 className="font-semibold text-mint-700 mb-3">
          Historial completo de mediciones
        </h3>
        {!measurements || measurements.length === 0 ? (
          <p className="text-sm text-mint-700">
            Aún no hay mediciones registradas para este usuario.
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="table min-w-[640px]">
              <thead>
                <tr>
                  <th>{t.common.fecha}</th>
                  <th>{t.measurements.peso}</th>
                  <th>{t.measurements.espalda}</th>
                  <th>{t.measurements.cintura}</th>
                  <th>{t.measurements.abdomen}</th>
                  <th>{t.measurements.cadera}</th>
                </tr>
              </thead>
              <tbody>
                {measurements.map((m) => {
                  const highlight = m.date >= weekStart && m.date <= weekEnd;
                  return (
                    <tr
                      key={m.id}
                      className={highlight ? "bg-mint-50 font-medium" : ""}
                    >
                      <td>{m.date}</td>
                      <td>{m.weight ?? "—"}</td>
                      <td>{m.back_cm ?? "—"}</td>
                      <td>{m.waist_cm ?? "—"}</td>
                      <td>{m.abdomen_cm ?? "—"}</td>
                      <td>{m.hip_cm ?? "—"}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
